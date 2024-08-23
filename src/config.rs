use crate::{commands, utils::config::selected_config_path};
use eyre::Context;
use serde::{de, Deserialize, Serialize};
use std::{
    fmt::Display,
    ops::{Deref, DerefMut},
    str::FromStr,
};
use zksync_ethers_rs::types::{zksync::url::SensitiveUrl, Address};

#[derive(Deserialize, Serialize, Clone)]
pub struct ZKSyncConfig {
    pub network: NetworkConfig,
    pub wallet: Option<WalletConfig>,
    pub db: Option<DatabaseConfig>,
    pub governance: GovernanceConfig,
    pub bridgehub: BridgehubConfig,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct NetworkConfig {
    pub l1_rpc_url: Option<String>,
    pub l1_chain_id: Option<u64>,
    pub l1_explorer_url: Option<String>,
    pub l2_rpc_url: String,
    pub l2_chain_id: Option<u64>,
    pub l2_explorer_url: Option<String>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct WalletConfig {
    pub address: Address,
    pub private_key: String,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct GovernanceConfig {
    pub address: Address,
    pub owner_private_key: String,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct BridgehubConfig {
    pub admin_private_key: Option<String>,
    pub owner_private_key: Option<String>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct DatabaseConfig {
    pub server: Database,
    pub prover: Database,
}

#[derive(Clone)]
pub struct Database {
    pub pool: sqlx::PgPool,
    pub url: SensitiveUrl,
}

impl<'de> Deserialize<'de> for Database {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let db_url = String::deserialize(deserializer)?;
        Ok(Database {
            pool: sqlx::PgPool::connect_lazy(&db_url).map_err(serde::de::Error::custom)?,
            url: SensitiveUrl::from_str(&db_url).map_err(serde::de::Error::custom)?,
        })
    }
}

impl Serialize for Database {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.url.expose_str().serialize(serializer)
    }
}

impl Deref for Database {
    type Target = sqlx::PgPool;

    fn deref(&self) -> &Self::Target {
        &self.pool
    }
}

impl DerefMut for Database {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.pool
    }
}

impl FromStr for Database {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Database {
            pool: sqlx::PgPool::connect_lazy(s)?,
            url: SensitiveUrl::from_str(s)?,
        })
    }
}

impl TryFrom<&str> for Database {
    type Error = eyre::Error;

    fn try_from(s: &str) -> Result<Database, Self::Error> {
        Self::from_str(s)
    }
}

// We implement this to have ToString also.
impl Display for Database {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.url.expose_str().fmt(f)
    }
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
