use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Configurations {
    timezone: Option<String>,
    database_url: Option<String>,
    database_name: Option<String>,
    servers: Option<Vec<ServerConfiguration>>,
}

impl Configurations {
    pub fn get_timezone(&self) -> String {
        self.get_string(self.timezone.clone(), "CHRONO_TIMEZONE", "Europe/Helsinki")
    }

    pub fn get_database_url(&self) -> String {
        self.get_string(self.database_url.clone(), "DATABASE_URL", "http://localhost:8086")
    }

    pub fn get_database_name(&self) -> String {
        self.get_string(self.database_name.clone(), "DATABASE_NAME", "bflogger")
    }

    pub fn get_servers(&self) -> Vec<ServerConfiguration> {
        if self.servers.is_none() {
            let server_addresses = dotenv::var("SERVER_ADDRS")
                .expect("Server addresses needed. Separate with comma (,) if multiple.");
            let addresses = server_addresses.split(",");

            let mut servers : Vec<ServerConfiguration> = Vec::new();
            for address in addresses {
                let mut game_ip_and_port = address.split(":");

                let game_ip = game_ip_and_port.next().unwrap();
                let rcon_port = game_ip_and_port.last().unwrap();

                servers.push(ServerConfiguration {
                    game_ip: game_ip.to_string(),
                    game_port: None,
                    rcon_port: rcon_port.to_string(),
                    unique_id: format!("{}:{}", game_ip, rcon_port),

                    update_interval: None,
                    max_retry_connection_interval: None,
                    retry_connection_step: None,
                    retry_connection_addition: None,
                })
            }

            return servers;
        }
        
        self.servers.as_ref().unwrap().clone()
    }

    fn get_string(&self, setting: Option<String>, key: &str, fallback: &str) -> String {
        if setting.is_none() {
            return dotenv::var(key).unwrap_or(fallback.to_string());
        }

        setting.unwrap()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfiguration {
    game_ip: String,
    game_port: Option<String>,
    rcon_port: String,
    unique_id: String,

    update_interval: Option<u64>,
    max_retry_connection_interval: Option<u64>,
    retry_connection_step: Option<i32>,
    retry_connection_addition: Option<u64>,
}

impl ServerConfiguration {
    pub fn get_game_ip_rcon_port(&self) -> String {
        format!("{}:{}", self.game_ip, self.rcon_port)
    }

    pub fn get_game_ip_port(&self, fallback: &str) -> String {
        if self.game_port.is_none() {
            return fallback.to_string();
        }

        format!("{}:{}", self.game_ip, self.game_port.as_ref().unwrap())
    }

    pub fn get_unique_id(&self) -> String {
        self.unique_id.clone()
    }
    
    pub fn get_server_info_interval(&self) -> u64 {
        self.get_u64_with_key_fallback(&self.update_interval, "SERVER_INFO_INTERVAL", 10000)
    }

    pub fn get_max_retry_connection_interval(&self) -> u64 {
        self.get_u64_with_fallback(&self.max_retry_connection_interval, 300)
    }

    pub fn get_retry_connection_step(&self) -> i32 {
        self.get_i32_with_fallback(&self.retry_connection_step, 10)
    }

    pub fn get_retry_connection_addition(&self) -> u64 {
        self.get_u64_with_fallback(&self.retry_connection_addition, 30)
    }

    fn get_i32_with_fallback(&self, setting: &Option<i32>, fallback: i32) -> i32 {
        if setting.is_none() {
            return fallback;
        }

        setting.unwrap()
    }

    fn get_u64_with_fallback(&self, setting: &Option<u64>, fallback: u64) -> u64 {
        if setting.is_none() {
            return fallback;
        }

        setting.unwrap()
    }

    fn get_u64_with_key_fallback(&self, setting: &Option<u64>, key: &str, fallback: u64) -> u64 {
        if setting.is_none() {
            return dotenv::var(key)
                .map(|var| var.parse::<u64>())
                .unwrap_or(Ok(fallback))
                .unwrap();
        }

        setting.unwrap()
    }
}
