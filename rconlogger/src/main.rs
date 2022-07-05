use std::sync::Arc;
use std::time::Duration;

use battlefield_rcon::bf4::Bf4Client;
use battlefield_rcon::bf4::ServerInfoError;
use anyhow::{anyhow};
use chrono::{DateTime, Utc};
use dotenv::dotenv;
use influxdb::Client;
use influxdb::InfluxDbWriteable;
use tokio::time::sleep;

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
}

async fn log_new_entry(client: &Client, bf4: &Bf4Client, addr: &String) -> anyhow::Result<()> {
    println!("Logging new server info entry for server {}", &addr);

    // Server info
    match bf4.server_info().await {
        Ok(data) => {
            // Let's write some data into a measurement called `serverinfo`
            let serverinfo_reading = ServerInfoReading {
                time: Utc::now(),
                server_name: data.server_name.to_string(),
                server_name_tag: data.server_name.to_string(),
                playercount: data.playercount,
                max_playercount: data.max_playercount,
                game_mode: data.game_mode.to_string(),
                map: data.map.to_string(),
                rounds_played: data.rounds_played,
                rounds_total: data.rounds_total,
                online_state: data.online_state.to_string(),
                ranked: data.ranked,
                punkbuster: data.punkbuster,
                has_gamepassword: data.has_gamepassword,
                server_uptime: data.server_uptime,
                roundtime: data.roundtime,
                game_ip_and_port: data.game_ip_and_port.to_string(),
                punkbuster_version: data.punkbuster_version.to_string(),
                join_queue_enabled: data.join_queue_enabled,
                region: data.region.to_string(),
                closest_ping_site: data.closest_ping_site.to_string(),
                country: data.country.to_string(),
                blaze_player_count: data.blaze_player_count,
                blaze_game_state: data.blaze_game_state.to_string(),
            };

            let write_result = client.query(&serverinfo_reading.into_query("serverinfo")).await;
            if let Err(err) = write_result {
                eprintln!("Error writing to db: {}", err)
            }

            Ok(())
        },
        Err(ServerInfoError::Rcon(rconerr)) => {
            return Err(anyhow!("RCON error: {:?}", rconerr))
        },
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let database_url = dotenv::var("DATABASE_URL").unwrap_or("http://localhost:8086".to_string());
    let database_name = dotenv::var("DATABASE_NAME").unwrap_or("bflogger".to_string());
    let interval: u64 = dotenv::var("SERVER_INFO_INTERVAL")
        .map(|var| var.parse::<u64>())
        .unwrap_or(Ok(10000))
        .unwrap();
    
    let server_addresses = dotenv::var("SERVER_ADDRS")
        .expect("Server addresses needed. Separate with comma (,) if multiple.");

    let mut jhs = Vec::new();
    let split = server_addresses.split(",");

    for s in split {
        let url = database_url.clone();
        let name = database_name.clone();
        let addr = String::from(s);
        jhs.push(tokio::spawn(async move {
            let max_connection_interval: u64 = 300;
            let mut connection_attempts: i32 = 1;
            let mut connection_interval: u64 = 0;
            let mut bf4: Arc<Bf4Client>;

            // Connection loop
            loop {
                let connection = Bf4Client::connect_restricted(
                    &addr,
                )
                .await;

                if connection.is_ok() {
                    bf4 = connection.unwrap();
                    println!("Connected to {}", &addr);
                    break;
                }

                println!("Connection failed {}", &addr);
                connection_attempts += 1;
                if connection_attempts % 10 == 0 {
                    connection_interval += 30;
                }
                if connection_interval >= max_connection_interval {
                    connection_interval = max_connection_interval;
                }

                sleep(Duration::from_secs(10)).await;
            }

            // Connect to database
            let client = Client::new(url, name);

            println!("Starting fetch loop for server {} with the interval of {}", &addr, interval);

            loop {
                match log_new_entry(&client, &bf4, &addr).await {
                    Err(err) => {
                        println!("Reconnecting loop started for {} because of {}", &addr, err);

                        // Reconnect loop
                        connection_attempts = 1;
                        connection_interval = 0;

                        loop {
                            println!("Attempt {} at trying to reconnect {}", connection_attempts, &addr);

                            let connection = Bf4Client::connect_restricted(
                                &addr,
                            )
                            .await;
            
                            if connection.is_ok() {
                                bf4 = connection.unwrap();
                                println!("Reconnected to {}", &addr);
                                sleep(Duration::from_secs(5)).await;
                                break;
                            }

                            println!("Reconnecting failed {}", &addr);
                            connection_attempts += 1;
                            if connection_attempts % 10 == 0 {
                                connection_interval += 30;
                            }
                            if connection_interval >= max_connection_interval {
                                connection_interval = max_connection_interval;
                            }
            
                            sleep(Duration::from_secs(connection_interval)).await;
                        }
                    },
                    Ok(_) => sleep(Duration::from_millis(interval)).await,
                };
                
            }
        }));

        sleep(Duration::from_millis(2000)).await;
    }

    // Wait for all our spawned tasks to finish.
    for jh in jhs.drain(..) {
        jh.await.unwrap()
    }
}
