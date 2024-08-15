use crate::{
    commands::config::create,
    config::{NetworkConfig, WalletConfig, ZKSyncConfig},
};
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};
use eyre::ContextCompat;
use std::{path::PathBuf, str::FromStr};
use zksync_ethers_rs::types::H160;

pub const SELECTED_CONFIG_FILE_NAME: &str = ".selected";

pub const DEFAULT_L1_RPC_URL: &str = "http://localhost:8545";
pub const DEFAULT_L2_RPC_URL: &str = "http://localhost:3030";
pub const DEFAULT_L2_EXPLORER_URL: &str = "http://localhost:3010";
pub const DEFAULT_L1_EXPLORER_URL: &str = "";
pub const DEFAULT_PRIVATE_KEY: &str =
    "0x7726827caac94a7f9e1b160f7ea819f172f7b6f9d2a97f992c38edeab82d4110";
// 0x36615Cf349d7F6344891B1e7CA7C72883F5dc049
pub const DEFAULT_ADDRESS: H160 = H160([
    0x36, 0x61, 0x5C, 0xf3, 0x49, 0xd7, 0xf6, 0x34, 0x48, 0x91, 0xb1, 0xe7, 0xca, 0x7c, 0x72, 0x88,
    0x3f, 0x5d, 0xc0, 0x48,
]);

pub const CONFIG_OVERRIDE_PROMPT_MSG: &str = "Config already exists. Do you want to overwrite it?";
pub const CONFIG_CREATE_PROMPT_MSG: &str = "This config does not exist. Do you want to create it?";
pub const CONFIG_EDIT_PROMPT_MSG: &str = "What config do you want to edit?";
pub const CONFIG_SET_PROMPT_MSG: &str = "What config do you want to set?";
pub const CONFIG_DELETE_PROMPT_MSG: &str = "Are you sure you want to delete this config?";
pub const CONFIG_SELECTION_TO_DELETE_PROMPT_MSG: &str = "What config do you want to delete?";
pub const CONFIG_TO_DISPLAY_PROMPT_MSG: &str = "What config do you want to see?";
pub const L1_RPC_URL_PROMPT_MSG: &str = "L1 RPC URL";
pub const L2_RPC_URL_PROMPT_MSG: &str = "L2 RPC URL";
pub const L2_EXPLORER_URL_PROMPT_MSG: &str = "L2 Explorer URL";
pub const L1_EXPLORER_URL_PROMPT_MSG: &str = "L1 Explorer URL";
pub const PRIVATE_KEY_PROMPT_MSG: &str = "Private key";
pub const ADDRESS_PROMPT_MSG: &str = "Address";

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
    Ok(configs_dir_path()?.join(&format!("{config_name}.toml")))
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
