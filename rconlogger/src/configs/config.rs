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
        assert_eq!(settings.get_server_info_interval(), 1337);

        let servers = settings.get_servers();
        assert_eq!(servers.len(), 1);
        assert_eq!(servers[0].get_game_ip_port(""), "127.0.0.1:25200".to_string());
        assert_eq!(servers[0].get_game_ip_rcon_port(), "127.0.0.1:47200".to_string());
        assert_eq!(servers[0].get_unique_id(), "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx".to_string());
    }
}
