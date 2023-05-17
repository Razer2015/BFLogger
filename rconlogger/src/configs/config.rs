use std::fs::File;
use std::io::Read;
use std::path::Path;
use thiserror::Error;
use crate::configs::config_model::Configurations;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to deserialize config.")]
    Serde(#[from] serde_yaml::Error),
    #[error("Failed to open config file")]
    Io(#[from] std::io::Error),
}

pub fn load_configurations(path: impl AsRef<Path>) -> Result<Configurations, ConfigError> {
    println!("Loading {}", path.as_ref().to_string_lossy());
    let mut file = File::open(path)?;
    let mut s = String::new();
    file.read_to_string(&mut s)?;
    let t: Configurations = serde_yaml::from_str(&s)?;

    Ok(t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_load_configurations() {
        let settings = load_configurations(format!("configs/{}.yaml", "test"))
            .expect("Failed to load configuration file.");

        println!("Configurations {:#?}", settings);

        assert_eq!(settings.get_timezone(), "Europe/Helsinki (test)");
        assert_eq!(settings.get_database_url(), "http://localhost:8086 (test)");
        assert_eq!(settings.get_database_name(), "bflogger (test)");

        let servers = settings.get_servers();
        assert_eq!(servers.len(), 2);
        assert_eq!(servers[0].get_game_ip_port(""), "127.0.0.1:25200".to_string());
        assert_eq!(servers[0].get_game_ip_rcon_port(), "127.0.0.1:47200".to_string());
        assert_eq!(servers[0].get_unique_id(), "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx".to_string());
        assert_eq!(servers[0].get_server_info_interval(), 1337);
        assert_eq!(servers[0].get_max_retry_connection_interval(), 1338);
        assert_eq!(servers[0].get_retry_connection_step(), 15);
        assert_eq!(servers[0].get_retry_connection_addition(), 1339);

        assert_eq!(servers[1].get_game_ip_port("0.0.0.0:10000"), "0.0.0.0:10000".to_string());
        assert_eq!(servers[1].get_game_ip_rcon_port(), "0.0.0.0:47201".to_string());
        assert_eq!(servers[1].get_unique_id(), "axxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx".to_string());
        assert_eq!(servers[1].get_server_info_interval(), 10000);
        assert_eq!(servers[1].get_max_retry_connection_interval(), 300);
        assert_eq!(servers[1].get_retry_connection_step(), 10);
        assert_eq!(servers[1].get_retry_connection_addition(), 30);
    }
}
