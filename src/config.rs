use crate::commands::config::{
    common::{selected_config_path, SELECTED_CONFIG_FILE_NAME},
    set,
};
use eyre::Context;
use serde::{Deserialize, Serialize};
use zksync_ethers_rs::types::Address;

#[derive(Deserialize, Serialize)]
pub struct ZKSyncConfig {
    pub network: NetworkConfig,
    pub wallet: Option<WalletConfig>,
}

#[derive(Deserialize, Serialize)]
pub struct NetworkConfig {
    pub l1_rpc_url: Option<String>,
    pub l1_explorer_url: Option<String>,
    pub l2_rpc_url: String,
    pub l2_explorer_url: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct WalletConfig {
    pub address: Address,
    pub private_key: String,
}

pub async fn try_load_selected_config() -> eyre::Result<Option<ZKSyncConfig>> {
    let config_path = selected_config_path()?;
    if !config_path.exists() {
        return Ok(None);
    }
    let config = std::fs::read_to_string(config_path).context("Failed to read config file")?;
    toml::from_str(&config)
        .context("Failed to parse config file")
        .map(Some)
}

pub async fn load_selected_config() -> eyre::Result<ZKSyncConfig> {
    let config_path = selected_config_path()?;
    if !config_path.exists() {
        println!("No config set, please select a config to set");
        set::run(set::Args {
            config_name: SELECTED_CONFIG_FILE_NAME.into(),
            set_config_interactively: true,
        })
        .await?;
    }
    let config = std::fs::read_to_string(config_path).context("Failed to read config file")?;
    toml::from_str(&config).context("Failed to parse config file")
}
