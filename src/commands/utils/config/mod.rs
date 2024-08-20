use crate::{
    commands::{self, config::EditConfigOpts},
    config::{BridgehubConfig, GovernanceConfig, NetworkConfig, WalletConfig, ZKSyncConfig},
};
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};
use eyre::ContextCompat;
use std::{path::PathBuf, str::FromStr};

pub mod default_values;
use default_values::{
    DEFAULT_ADDRESS, DEFAULT_CONTRACT_ADDRESS, DEFAULT_L1_EXPLORER_URL, DEFAULT_L1_RPC_URL,
    DEFAULT_L2_EXPLORER_URL, DEFAULT_L2_RPC_URL, DEFAULT_PRIVATE_KEY,
};
pub mod messages;
use messages::{
    ADDRESS_PROMPT_MSG, CONFIG_CREATE_PROMPT_MSG, CONFIG_EDIT_PROMPT_MSG,
    CONTRACTS_BRIDGEHUB_ADMIN_PRIVATE_KEY_PROMPT_MSG,
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
        Box::pin(async {
            commands::config::start(commands::config::Command::Create { config_name }).await
        })
        .await
    } else {
        println!("Aborted");
        Ok(())
    }
}

pub fn selected_config_path() -> eyre::Result<PathBuf> {
    Ok(configs_dir_path()?.join(SELECTED_CONFIG_FILE_NAME))
}

pub fn edit_config_by_name_interactively(
    config_path: &PathBuf,
) -> eyre::Result<(ZKSyncConfig, bool)> {
    let existing_config: ZKSyncConfig = toml::from_str(&std::fs::read_to_string(config_path)?)?;
    let set_new_config = config_to_edit_is_set(&existing_config)?;
    let new_config = edit_existing_config_interactively(existing_config)?;
    Ok((new_config, set_new_config))
}

pub fn edit_config_by_name_with_args(
    config_path: &PathBuf,
    opts: EditConfigOpts,
) -> eyre::Result<(ZKSyncConfig, bool)> {
    let existing_config: ZKSyncConfig = toml::from_str(&std::fs::read_to_string(config_path)?)?;
    let set_new_config = config_to_edit_is_set(&existing_config)?;
    let new_config = edit_existing_config_non_interactively(existing_config, opts)?;
    Ok((new_config, set_new_config))
}

pub fn edit_config_interactively() -> eyre::Result<(ZKSyncConfig, PathBuf, bool)> {
    let config_path = config_path_interactive_selection(CONFIG_EDIT_PROMPT_MSG)?;
    let existing_config: ZKSyncConfig =
        toml::from_str(&std::fs::read_to_string(config_path.clone())?)?;
    let set_new_config = config_to_edit_is_set(&existing_config)?;
    let new_config = edit_existing_config_interactively(existing_config)?;
    Ok((new_config, config_path, set_new_config))
}

pub fn config_to_edit_is_set(existing_config: &ZKSyncConfig) -> eyre::Result<bool> {
    let selected_config_path = selected_config_path()?;
    if !selected_config_path.exists() {
        return Ok(false);
    }
    let selected_config: ZKSyncConfig =
        toml::from_str(&std::fs::read_to_string(selected_config_path)?)?;
    Ok(&selected_config == existing_config)
}

pub async fn set_new_config_if_needed(
    set_new_config: bool,
    config_path: PathBuf,
) -> eyre::Result<()> {
    if set_new_config {
        Box::pin(async {
            commands::config::start(commands::config::Command::Set {
                config_name: Some(
                    config_path
                        .file_stem()
                        .context("There's no file name")?
                        .to_os_string()
                        .into_string()
                        .map_err(|e| eyre::eyre!("Invalid file name: {:?}", e.into_string()))?,
                ),
            })
            .await
        })
        .await?
    }
    Ok(())
}

pub fn edit_existing_config_interactively(
    existing_config: ZKSyncConfig,
) -> eyre::Result<ZKSyncConfig> {
    let config = ZKSyncConfig {
        network: NetworkConfig {
            l1_rpc_url: prompt(
                L1_RPC_URL_PROMPT_MSG,
                existing_config
                    .network
                    .l1_rpc_url
                    .unwrap_or(DEFAULT_L1_RPC_URL.into()),
            )
            .ok(),
            l2_rpc_url: prompt(L2_RPC_URL_PROMPT_MSG, existing_config.network.l2_rpc_url)?,
            l2_explorer_url: prompt(
                L2_EXPLORER_URL_PROMPT_MSG,
                existing_config
                    .network
                    .l2_explorer_url
                    .unwrap_or(DEFAULT_L2_EXPLORER_URL.into()),
            )
            .ok(),
            l1_explorer_url: prompt(
                L1_EXPLORER_URL_PROMPT_MSG,
                existing_config
                    .network
                    .l1_explorer_url
                    .unwrap_or(DEFAULT_L1_EXPLORER_URL.into()),
            )
            .ok(),
        },
        wallet: Some(WalletConfig {
            private_key: prompt(
                PRIVATE_KEY_PROMPT_MSG,
                existing_config
                    .wallet
                    .as_ref()
                    .map(|w| w.private_key.clone())
                    .unwrap_or(DEFAULT_PRIVATE_KEY.into()),
            )?,
            address: prompt(
                ADDRESS_PROMPT_MSG,
                existing_config
                    .wallet
                    .as_ref()
                    .map(|w| w.address)
                    .unwrap_or(DEFAULT_ADDRESS),
            )?,
        }),
        governance: GovernanceConfig {
            address: prompt(
                CONTRACTS_GOVERNANCE_PROMPT_MSG,
                existing_config.governance.address,
            )?,
            owner_private_key: prompt(
                DEFAULT_PRIVATE_KEY,
                existing_config.governance.owner_private_key,
            )?,
        },
        bridgehub: BridgehubConfig {
            admin_private_key: prompt(
                CONTRACTS_BRIDGEHUB_ADMIN_PRIVATE_KEY_PROMPT_MSG,
                existing_config
                    .bridgehub
                    .admin_private_key
                    .unwrap_or(DEFAULT_PRIVATE_KEY.into()),
            )
            .ok(),
            owner_private_key: prompt(
                CONTRACTS_BRIDGEHUB_OWNER_PRIVATE_KEY_PROMPT_MSG,
                existing_config
                    .bridgehub
                    .owner_private_key
                    .unwrap_or(DEFAULT_PRIVATE_KEY.into()),
            )
            .ok(),
        },
    };
    Ok(config)
}

pub fn edit_existing_config_non_interactively(
    existing_config: ZKSyncConfig,
    opts: EditConfigOpts,
) -> eyre::Result<ZKSyncConfig> {
    let config = ZKSyncConfig {
        network: NetworkConfig {
            l1_rpc_url: opts.l1_rpc_url.or(existing_config.network.l1_rpc_url),
            l2_rpc_url: opts
                .l2_rpc_url
                .unwrap_or(existing_config.network.l2_rpc_url),
            l2_explorer_url: opts
                .l2_explorer_url
                .or(existing_config.network.l2_explorer_url),
            l1_explorer_url: opts
                .l1_explorer_url
                .or(existing_config.network.l1_explorer_url),
        },
        wallet: existing_config
            .wallet
            .map(|existing_wallet_config| WalletConfig {
                private_key: opts
                    .private_key
                    .unwrap_or(existing_wallet_config.private_key),
                address: opts.address.unwrap_or(existing_wallet_config.address),
            }),
        governance: GovernanceConfig {
            address: opts
                .governance
                .unwrap_or(existing_config.governance.address),
            owner_private_key: opts
                .governance_owner
                .unwrap_or(existing_config.governance.owner_private_key),
        },
        bridgehub: BridgehubConfig {
            admin_private_key: opts
                .bridgehub_admin
                .or(existing_config.bridgehub.admin_private_key),
            owner_private_key: opts
                .bridgehub_owner
                .or(existing_config.bridgehub.owner_private_key),
        },
    };
    Ok(config)
}
