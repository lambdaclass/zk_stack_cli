use crate::{commands, utils::config::selected_config_path};
use eyre::Context;
use serde::{Deserialize, Serialize};
use zksync_ethers_rs::types::Address;

#[derive(Deserialize, Serialize, PartialEq, Clone)]
pub struct ZKSyncConfig {
    pub network: NetworkConfig,
    pub wallet: Option<WalletConfig>,
    pub governance: GovernanceConfig,
    pub bridgehub: BridgehubConfig,
}

#[derive(Deserialize, Serialize, PartialEq, Clone)]
pub struct NetworkConfig {
    pub l1_rpc_url: Option<String>,
    pub l1_explorer_url: Option<String>,
    pub l2_rpc_url: String,
    pub l2_explorer_url: Option<String>,
}

#[derive(Deserialize, Serialize, PartialEq, Clone)]
pub struct WalletConfig {
    pub address: Address,
    pub private_key: String,
}

#[derive(Deserialize, Serialize, PartialEq, Clone)]
pub struct GovernanceConfig {
    pub address: Address,
    pub owner_private_key: String,
}

#[derive(Deserialize, Serialize, PartialEq, Clone)]
pub struct BridgehubConfig {
    pub admin_private_key: Option<String>,
    pub owner_private_key: Option<String>,
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
        commands::config::Command::Set { config_name: None }
            .run()
            .await?;
    }
    let config = std::fs::read_to_string(config_path).context("Failed to read config file")?;
    toml::from_str(&config).context("Failed to parse config file")
}
