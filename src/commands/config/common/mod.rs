use crate::{
    commands::config::create,
    config::{BridgehubConfig, GovernanceConfig, NetworkConfig, WalletConfig, ZKSyncConfig},
};
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};
use eyre::ContextCompat;
use std::{path::PathBuf, str::FromStr};

pub(crate) mod default_values;
use default_values::{
    DEFAULT_ADDRESS, DEFAULT_CONTRACT_ADDRESS, DEFAULT_L1_EXPLORER_URL, DEFAULT_L1_RPC_URL,
    DEFAULT_L2_EXPLORER_URL, DEFAULT_L2_RPC_URL, DEFAULT_PRIVATE_KEY,
};
pub(crate) mod messages;
use messages::{
    ADDRESS_PROMPT_MSG, CONFIG_CREATE_PROMPT_MSG, CONTRACTS_BRIDGEHUB_ADMIN_PRIVATE_KEY_PROMPT_MSG,
    CONTRACTS_BRIDGEHUB_OWNER_PRIVATE_KEY_PROMPT_MSG, CONTRACTS_GOVERNANCE_PRIVATE_KEY_PROMPT_MSG,
    CONTRACTS_GOVERNANCE_PROMPT_MSG, L1_EXPLORER_URL_PROMPT_MSG, L1_RPC_URL_PROMPT_MSG,
    L2_EXPLORER_URL_PROMPT_MSG, L2_RPC_URL_PROMPT_MSG, PRIVATE_KEY_PROMPT_MSG,
};

pub const SELECTED_CONFIG_FILE_NAME: &str = ".selected";

pub fn configs_dir_path() -> eyre::Result<std::path::PathBuf> {
    let configs_dir_path = dirs::config_dir()
        .ok_or_else(|| eyre::eyre!("Could not find user's config directory"))?
        .join("zks-cli")
        .join("configs");
    if !configs_dir_path.exists() {
        std::fs::create_dir_all(&configs_dir_path)?;
    }
    Ok(configs_dir_path)
}

pub fn config_path(config_name: &str) -> eyre::Result<std::path::PathBuf> {
    Ok(configs_dir_path()?.join(format!("{config_name}.toml")))
}

pub fn prompt<T>(prompt: &str, default: T) -> eyre::Result<T>
where
    T: Clone + ToString + FromStr,
    <T as FromStr>::Err: ToString,
{
    Input::<T>::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .default(default)
        .show_default(true)
        .interact_text()
        .map_err(Into::into)
}

pub fn confirm(prompt: &str) -> eyre::Result<bool> {
    Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .show_default(true)
        .default(false)
        .interact()
        .map_err(Into::into)
}

pub fn config_file_names() -> eyre::Result<Vec<String>> {
    let config_file_names_with_selected_config_file = std::fs::read_dir(configs_dir_path()?)?
        .map(|entry| {
            entry
                .map_err(Into::into)
                .and_then(|entry| {
                    entry
                        .file_name()
                        .into_string()
                        .map_err(|e| eyre::eyre!("Invalid file name: {:?}", e.into_string()))
                })
                .map(|file_name| file_name.replace(".toml", ""))
        })
        .collect::<Result<Vec<String>, eyre::Error>>()?;
    let config_file_names = config_file_names_with_selected_config_file
        .into_iter()
        .filter(|file_name| file_name != SELECTED_CONFIG_FILE_NAME)
        .collect();
    Ok(config_file_names)
}

pub fn config_path_interactive_selection(prompt: &str) -> eyre::Result<PathBuf> {
    let configs = config_file_names()?;
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .items(&configs)
        .interact()?;
    config_path(configs.get(selection).context("No config selected")?)
}

pub fn prompt_zksync_config() -> eyre::Result<ZKSyncConfig> {
    let prompted_config = ZKSyncConfig {
        network: NetworkConfig {
            l1_rpc_url: prompt(L1_RPC_URL_PROMPT_MSG, DEFAULT_L1_RPC_URL.into()).ok(),
            l2_rpc_url: prompt(L2_RPC_URL_PROMPT_MSG, DEFAULT_L2_RPC_URL.into())?,
            l2_explorer_url: prompt(L2_EXPLORER_URL_PROMPT_MSG, DEFAULT_L2_EXPLORER_URL.into())
                .ok(),
            l1_explorer_url: prompt(L1_EXPLORER_URL_PROMPT_MSG, DEFAULT_L1_EXPLORER_URL.into())
                .ok(),
        },
        wallet: Some(WalletConfig {
            private_key: prompt(PRIVATE_KEY_PROMPT_MSG, DEFAULT_PRIVATE_KEY.into())?,
            address: prompt(ADDRESS_PROMPT_MSG, DEFAULT_ADDRESS)?,
        }),
        governance: GovernanceConfig {
            address: prompt(CONTRACTS_GOVERNANCE_PROMPT_MSG, DEFAULT_CONTRACT_ADDRESS)?,
            owner_private_key: prompt(
                CONTRACTS_GOVERNANCE_PRIVATE_KEY_PROMPT_MSG,
                DEFAULT_PRIVATE_KEY.into(),
            )?,
        },
        bridgehub: BridgehubConfig {
            admin_private_key: prompt(
                CONTRACTS_BRIDGEHUB_ADMIN_PRIVATE_KEY_PROMPT_MSG,
                DEFAULT_PRIVATE_KEY.into(),
            )
            .ok(),
            owner_private_key: prompt(
                CONTRACTS_BRIDGEHUB_OWNER_PRIVATE_KEY_PROMPT_MSG,
                DEFAULT_PRIVATE_KEY.into(),
            )
            .ok(),
        },
    };
    Ok(prompted_config)
}

pub async fn confirm_config_creation(config_name: String) -> eyre::Result<()> {
    let create_confirmation = confirm(CONFIG_CREATE_PROMPT_MSG)?;
    if create_confirmation {
        create::run(create::Args { config_name }).await
    } else {
        println!("Aborted");
        Ok(())
    }
}

pub fn selected_config_path() -> eyre::Result<PathBuf> {
    Ok(configs_dir_path()?.join(SELECTED_CONFIG_FILE_NAME))
}
