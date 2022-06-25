use std::time::Duration;

use battlelog::server_snapshot;
use chrono::{DateTime, Utc};
use dotenv::dotenv;
use influxdb::Client;
use influxdb::InfluxDbWriteable;
use tokio::time::sleep;

#[derive(InfluxDbWriteable)]
struct SnapshotReading {
    time: DateTime<Utc>,
    #[influxdb(tag)]
    server_guid: String,
    game_id: u64,
    #[influxdb(tag)]
    game_mode: String,
    map_variant: u8,
    #[influxdb(tag)]
    current_map: String,
    current_map_name: String,
    max_players: u8,
    waiting_players: u8,
    players: u16,
    round_time: u32,
    default_round_time_multiplier: u32,

    #[influxdb(tag)]
    round_running: bool,

    round_running_val: bool,

    defender_team: Option<u8>,
    defender_bases: Option<u8>,
    defender_bases_max: Option<u8>,
    defender_attacker: Option<u8>,

    attacker_team: Option<u8>,
    attacker_tickets: Option<u16>,
    attacker_tickets_max: Option<u16>,
    attacker_attacker: Option<u8>,
}

async fn log_new_entry(client: &Client, server_guid: &String) {
    println!("Logging new entry for server guid {}", &server_guid);

    match server_snapshot(&server_guid).await {
        Ok(data) => {
            // Let's write some data into a measurement called `snapshot`
            let mut snapshot_reading = SnapshotReading {
                time: Utc::now(),
                server_guid: server_guid.to_string(),
                game_id: data.snapshot.game_id,
                game_mode: data.snapshot.game_mode.to_string(),
                map_variant: data.snapshot.map_variant,
                current_map: data
                    .snapshot
                    .current_map
                    .split('/')
                    .last()
                    .unwrap_or("")
                    .to_string(),
                current_map_name: data
                    .snapshot
                    .current_map
                    .split('/')
                    .last()
                    .unwrap_or("")
                    .to_string(),
                max_players: data.snapshot.max_players,
                waiting_players: data.snapshot.waiting_players,
                players: data.snapshot.get_players_count(),
                round_time: data.snapshot.round_time,
                default_round_time_multiplier: data.snapshot.default_round_time_multiplier,

                round_running: data.snapshot.rush.is_some(),

                round_running_val: data.snapshot.rush.is_some(),

                defender_team: None,
                defender_bases: None,
                defender_bases_max: None,
                defender_attacker: None,

                attacker_team: None,
                attacker_tickets: None,
                attacker_tickets_max: None,
                attacker_attacker: None,
            };

            if data.snapshot.rush.is_some() {
                let rush = data.snapshot.rush.unwrap();
                let defenders = rush.defenders;

                snapshot_reading.defender_team = Some(defenders.team);
                snapshot_reading.defender_bases = Some(defenders.bases);
                snapshot_reading.defender_bases_max = Some(defenders.bases_max);
                snapshot_reading.defender_attacker = Some(defenders.attacker);

                let attackers = rush.attackers;
                snapshot_reading.attacker_team = Some(attackers.team);
                snapshot_reading.attacker_tickets = Some(attackers.tickets);
                snapshot_reading.attacker_tickets_max = Some(attackers.tickets_max);
                snapshot_reading.attacker_attacker = Some(attackers.attacker);
            }

            let write_result = client.query(&snapshot_reading.into_query("snapshot")).await;
            if let Err(err) = write_result {
                eprintln!("Error writing to db: {}", err)
            }
        }
        Err(_) => {}
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let database_url = dotenv::var("DATABASE_URL").unwrap_or("http://localhost:8086".to_string());
    let database_name = dotenv::var("DATABASE_NAME").unwrap_or("bflogger".to_string());
    let interval: u64 = dotenv::var("INTERVAL")
        .map(|var| var.parse::<u64>())
        .unwrap_or(Ok(30000))
        .unwrap();
    
    let server_guids = dotenv::var("SERVER_GUID")
        .expect("Server guid(s) needed. Separate with comma (,) if multiple.");

    let mut jhs = Vec::new();
    let split = server_guids.split(",");

    for s in split {
        let url = database_url.clone();
        let name = database_name.clone();
        let guid = String::from(s);
        jhs.push(tokio::spawn(async move {
            // Connect to database
            let client = Client::new(url, name);

            println!("Starting fetch loop for server guid {} with the interval of {}", &guid, interval);

            loop {
                log_new_entry(&client, &guid).await;
                sleep(Duration::from_millis(interval)).await;
            }
        }));

        sleep(Duration::from_millis(2000)).await;
    }

    // Wait for all our spawned tasks to finish.
    for jh in jhs.drain(..) {
        jh.await.unwrap()
    }
}
