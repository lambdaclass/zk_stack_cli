use serde::Deserialize;

#[derive(Deserialize)]
pub struct ZKSyncConfig {
    pub l1_rpc_url: String,
    pub l1_explorer_url: Option<String>,
    pub l2_rpc_url: String,
    pub l2_explorer_url: Option<String>,
}

pub fn config_path() -> String {
    format!("{}/etc/config.toml", env!("CARGO_MANIFEST_DIR"))
}

pub fn load_config() -> ZKSyncConfig {
    let config = std::fs::read_to_string(config_path()).expect("Failed to read config file");
    toml::from_str(&config).expect("Failed to parse config file")
}
