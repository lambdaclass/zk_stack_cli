use eyre::Context;
use serde::Deserialize;
use zksync_ethers_rs::types::Address;

#[derive(Deserialize)]
pub struct ZKSyncConfig {
    pub network: NetworkConfig,
    pub wallet: Option<WalletConfig>,
}

#[derive(Deserialize)]
pub struct NetworkConfig {
    pub l1_rpc_url: Option<String>,
    pub l1_explorer_url: Option<String>,
    pub l2_rpc_url: String,
    pub l2_explorer_url: Option<String>,
}

#[derive(Deserialize)]
pub struct WalletConfig {
    pub address: Address,
    pub private_key: String,
}

pub fn config_path() -> String {
    format!("{}/etc/config.toml", env!("CARGO_MANIFEST_DIR"))
}

pub fn load_config() -> eyre::Result<ZKSyncConfig> {
    let config = std::fs::read_to_string(config_path()).context("Failed to read config file")?;
    toml::from_str(&config).context("Failed to parse config file")
}
