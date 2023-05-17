#[macro_use]
extern crate log;

use std::sync::Arc;
use std::time::Duration;

use battlefield_rcon::bf4::Bf4Client;
use battlefield_rcon::bf4::ServerInfoError;
use anyhow::{anyhow};
use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use configs::config_model::Configurations;
use configs::config_model::ServerConfiguration;
use dotenv::dotenv;
use influxdb::Client;
use influxdb::InfluxDbWriteable;
use tokio::task::JoinHandle;
use tokio::time::sleep;

mod configs;
mod logging;

#[derive(InfluxDbWriteable)]
struct ServerInfoReading {
    time: DateTime<Utc>,
    server_name: String,
    #[influxdb(tag)]
    server_name_tag: String,
    playercount: i32,
    max_playercount: i32,
    game_mode: String,
    map: String,
    rounds_played: i32,
    rounds_total: i32,
    online_state: String,
    ranked: bool,
    punkbuster: bool,
    has_gamepassword: bool,
    server_uptime: i32,
    roundtime: i32,
    #[influxdb(tag)]
    game_ip_and_port: String,
    punkbuster_version: String,
    join_queue_enabled: bool,
    region: String,
    closest_ping_site: String,
    country: String,
    blaze_player_count: i32,
    blaze_game_state: String,
    #[influxdb(tag)]
    unique_id: String,
}

async fn log_new_entry(client: &Client, bf4: &Bf4Client, addr: &String, server: &ServerConfiguration) -> anyhow::Result<()> {
    info!("Logging new server info entry for server {}", &addr);

    // Server info
    match bf4.server_info().await {
        Ok(data) => {
            let mut game_ip_port = data.game_ip_and_port.to_string();
            if game_ip_port.is_empty() {
                game_ip_port = server.get_game_ip_port(&addr);
            }

            // Let's write some data into a measurement called `serverinfo`
            let serverinfo_reading = ServerInfoReading {
                time: Utc::now(),
                server_name: data.server_name.to_string(),
                server_name_tag: data.server_name.to_string(),
                playercount: data.playercount,
                max_playercount: data.max_playercount,
                game_mode: data.game_mode.to_string(),
                map: data.map_original.to_string(),
                rounds_played: data.rounds_played,
                rounds_total: data.rounds_total,
                online_state: data.online_state.to_string(),
                ranked: data.ranked,
                punkbuster: data.punkbuster,
                has_gamepassword: data.has_gamepassword,
                server_uptime: data.server_uptime,
                roundtime: data.roundtime,
                game_ip_and_port: game_ip_port,
                punkbuster_version: data.punkbuster_version.to_string(),
                join_queue_enabled: data.join_queue_enabled,
                region: data.region.to_string(),
                closest_ping_site: data.closest_ping_site.to_string(),
                country: data.country.to_string(),
                blaze_player_count: data.blaze_player_count,
                blaze_game_state: data.blaze_game_state.to_string(),
                unique_id: server.get_unique_id().to_string(),
            };

            let write_result = client.query(&serverinfo_reading.into_query("serverinfo")).await;
            if let Err(err) = write_result {
                error!("Error writing to db: {}", err)
            }

            Ok(())
        },
        Err(ServerInfoError::Rcon(rconerr)) => {
            return Err(anyhow!("RCON error: {:?}", rconerr))
        },
    }
}

fn get_timezone(configurations: &Configurations) -> Tz {
    let timezone = configurations.get_timezone();
    timezone.parse().unwrap()
}

fn serverinfo_task(server: ServerConfiguration) -> JoinHandle<()> {
    let config = configs::config::load_configurations(format!("configs/{}.yaml", "production"))
        .expect("Failed to load configurations file.");

    let database_url = config.get_database_url();
    let database_name = config.get_database_name();

    let url = database_url.clone();
    let name = database_name.clone();

    tokio::spawn(async move {
        let server_info_update_interval: u64 = server.get_server_info_interval();

        let max_retry_connection_interval: u64 = server.get_max_retry_connection_interval();
        let mut connection_attempts: i32 = 1;
        let mut connection_interval: u64 = 0;
        let mut bf4: Arc<Bf4Client>;

        // Connection loop
        loop {
            let connection = Bf4Client::connect_restricted(
                &server.get_game_ip_rcon_port(), false,
            )
            .await;

            if connection.is_ok() {
                bf4 = connection.unwrap();
                info!("Connected to {}", &server.get_game_ip_rcon_port());
                break;
            }

            error!("Connection failed {} - {:#?}", &server.get_game_ip_rcon_port(), connection.unwrap_err());
            connection_attempts += 1;
            if connection_attempts % server.get_retry_connection_step() == 0 {
                connection_interval += server.get_retry_connection_addition();
            }
            if connection_interval >= max_retry_connection_interval {
                connection_interval = max_retry_connection_interval;
            }

            sleep(Duration::from_secs(connection_interval)).await;
        }

        // Connect to database
        let client = Client::new(url, name);

        info!("Starting fetch loop for server {} with the server_info_update_interval of {}", &server.get_game_ip_rcon_port(), server_info_update_interval);

        loop {
            match log_new_entry(&client, &bf4, &server.get_game_ip_rcon_port(), &server).await {
                Err(err) => {
                    warn!("Reconnecting loop started for {} because of {}", &server.get_game_ip_rcon_port(), err);

                    // Reconnect loop
                    connection_attempts = 1;
                    connection_interval = 0;

                    loop {
                        info!("Attempt {} at trying to reconnect {}", connection_attempts, &server.get_game_ip_rcon_port());

                        let connection = Bf4Client::connect_restricted(
                            &server.get_game_ip_rcon_port(), false,
                        )
                        .await;
        
                        if connection.is_ok() {
                            bf4 = connection.unwrap();
                            info!("Reconnected to {}", &server.get_game_ip_rcon_port());
                            sleep(Duration::from_secs(5)).await;
                            break;
                        }

                        error!("Reconnecting failed {} - {:#?}", &server.get_game_ip_rcon_port(), connection.unwrap_err());
                        connection_attempts += 1;
                        if connection_attempts % server.get_retry_connection_step() == 0 {
                            connection_interval += server.get_retry_connection_addition();
                        }
                        if connection_interval >= max_retry_connection_interval {
                            connection_interval = max_retry_connection_interval;
                        }
        
                        sleep(Duration::from_secs(connection_interval)).await;
                    }
                },
                Ok(_) => sleep(Duration::from_millis(server_info_update_interval)).await,
            };
            
        }
    })
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    logging::init_logging();

    info!("BFLogger starting");

    let config = configs::config::load_configurations(format!("configs/{}.yaml", "production"))
        .expect("Failed to load configurations file.");

    info!("Using time zone: {}", get_timezone(&config).name());
    
    let mut jhs = Vec::new();
    let servers = config.get_servers();

    for server in servers {
        jhs.push(tokio::spawn(async move {
            loop {
                let res = serverinfo_task(server.clone()).await;
                match res {
                    Ok(_output) => { break; },
                    Err(err) if err.is_panic() => { 
                        /* handle panic in task, e.g. by going around loop to restart task */
                        error!("{}", err);
                        sleep(Duration::from_millis(2000)).await;
                     },
                    Err(err) => { 
                        /* handle other errors (mainly runtime shutdown) */
                        error!("{}", err);
                        sleep(Duration::from_millis(2000)).await;
                    },
                }
            }
        }));
    }

    // Wait for all our spawned tasks to finish.
    for jh in jhs.drain(..) {
        jh.await.unwrap()
    }
}
