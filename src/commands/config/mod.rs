use crate::{
    commands::{
        self,
        config::common::{
            config_path, config_path_interactive_selection, confirm_config_creation,
            default_values::{
                DEFAULT_ADDRESS, DEFAULT_L1_EXPLORER_URL, DEFAULT_L1_RPC_URL,
                DEFAULT_L2_EXPLORER_URL, DEFAULT_PRIVATE_KEY,
            },
            messages::{
                ADDRESS_PROMPT_MSG, CONFIG_EDIT_PROMPT_MSG,
                CONTRACTS_BRIDGEHUB_ADMIN_PRIVATE_KEY_PROMPT_MSG,
                CONTRACTS_BRIDGEHUB_OWNER_PRIVATE_KEY_PROMPT_MSG, CONTRACTS_GOVERNANCE_PROMPT_MSG,
                L1_EXPLORER_URL_PROMPT_MSG, L1_RPC_URL_PROMPT_MSG, L2_EXPLORER_URL_PROMPT_MSG,
                L2_RPC_URL_PROMPT_MSG, PRIVATE_KEY_PROMPT_MSG,
            },
            prompt, selected_config_path,
        },
    },
    config::{BridgehubConfig, GovernanceConfig, NetworkConfig, WalletConfig, ZKSyncConfig},
};
use clap::{Parser, Subcommand};
use common::{
    config_file_names, confirm,
    messages::{
        CONFIG_DELETE_PROMPT_MSG, CONFIG_OVERRIDE_PROMPT_MSG,
        CONFIG_SELECTION_TO_DELETE_PROMPT_MSG, CONFIG_SET_PROMPT_MSG, CONFIG_TO_DISPLAY_PROMPT_MSG,
    },
    prompt_zksync_config,
};
use eyre::ContextCompat;
use std::path::PathBuf;
use zksync_ethers_rs::types::Address;

pub(crate) mod common;

#[allow(clippy::large_enum_variant)]
#[derive(Subcommand, PartialEq)]
pub(crate) enum Command {
    #[clap(about = "Edit an existing config.")]
    Edit {
        config_name: Option<String>,
        #[command(flatten)]
        opts: EditConfigOpts,
    },
    #[clap(about = "Create a new config.")]
    Create { config_name: String },
    #[clap(about = "Set the config to use.")]
    Set { config_name: Option<String> },
    #[clap(about = "Display a config.")]
    Display { config_name: Option<String> },
    #[clap(about = "List all configs.")]
    List,
    #[clap(about = "Delete a config.")]
    Delete { config_name: Option<String> },
}

#[derive(Parser, PartialEq)]
pub struct EditConfigOpts {
    #[arg(long, requires = "config_name", required = false)]
    l1_rpc_url: Option<String>,
    #[arg(long, requires = "config_name", required = false)]
    l2_rpc_url: Option<String>,
    #[arg(long, requires = "config_name", required = false)]
    l2_explorer_url: Option<String>,
    #[arg(long, requires = "config_name", required = false)]
    l1_explorer_url: Option<String>,
    #[arg(long, requires = "config_name", required = false)]
    private_key: Option<String>,
    #[arg(long, requires = "config_name", required = false)]
    address: Option<Address>,
    #[arg(long, requires = "config_name", required = false)]
    governance: Option<Address>,
    #[arg(long, requires = "config_name", required = false)]
    governance_owner: Option<String>,
    #[arg(long, requires = "config_name", required = false)]
    bridgehub_admin: Option<String>,
    #[arg(long, requires = "config_name", required = false)]
    bridgehub_owner: Option<String>,
}

pub(crate) async fn start(cmd: Command) -> eyre::Result<()> {
    match cmd {
        Command::Edit { config_name, opts } => {
            let (new_config, config_path, set_new_config) =
                if let Some(ref config_name) = config_name {
                    let config_path = config_path(config_name)?;
                    if !config_path.exists() {
                        return confirm_config_creation(config_name.clone()).await;
                    }
                    let (new_config, set_new_config) = if opts.l1_explorer_url.is_none()
                        && opts.l1_rpc_url.is_none()
                        && opts.l2_explorer_url.is_none()
                        && opts.l2_rpc_url.is_none()
                        && opts.private_key.is_none()
                        && opts.address.is_none()
                    {
                        edit_config_by_name_interactively(&config_path)?
                    } else {
                        edit_config_by_name_with_args(&config_path, opts)?
                    };
                    (new_config, config_path, set_new_config)
                } else {
                    edit_config_interactively()?
                };
            let toml_config = toml::to_string_pretty(&new_config)?;
            std::fs::write(&config_path, &toml_config)?;
            set_new_config_if_needed(set_new_config, config_path.clone()).await?;
            println!("Config updated at: {}", config_path.display());
            println!("\n{toml_config}");
        }
        Command::Create { config_name } => {
            let config_path = config_path(&config_name)?;
            if config_path.exists() {
                let override_confirmation = confirm(CONFIG_OVERRIDE_PROMPT_MSG)?;
                if !override_confirmation {
                    println!("Aborted");
                    return Ok::<(), eyre::Error>(());
                }
            }
            let config = prompt_zksync_config()?;
            let toml_config = toml::to_string_pretty(&config)?;
            println!(
                "Config created at: {}\n{toml_config}",
                config_path.display()
            );
            std::fs::write(config_path, toml_config)?;
        }
        Command::Set { config_name } => {
            let config_path_to_select = if let Some(config_name) = config_name {
                let config_path_to_select = config_path(&config_name)?;
                if !config_path_to_select.exists() {
                    return confirm_config_creation(config_name).await;
                }
                config_path_to_select
            } else {
                config_path_interactive_selection(CONFIG_SET_PROMPT_MSG)?
            };
            let selected_config = std::fs::read_to_string(config_path_to_select)?;
            std::fs::write(selected_config_path()?, &selected_config)?;
            println!("Config \"{selected_config}\" set");
        }
        Command::Display { config_name } => {
            let config_to_display_path = if let Some(config_name) = config_name {
                let config_to_display_path = config_path(&config_name)?;
                if !config_to_display_path.exists() {
                    return confirm_config_creation(config_name).await;
                }
                config_to_display_path
            } else {
                config_path_interactive_selection(CONFIG_TO_DISPLAY_PROMPT_MSG)?
            };
            println!("Config at: {}", config_to_display_path.display());
            println!();
            println!("{}", std::fs::read_to_string(config_to_display_path)?);
        }
        Command::List => {
            let config_file_names = config_file_names()?;
            if config_file_names.is_empty() {
                println!("No configs found");
                return Ok(());
            }
            println!("Configs:");
            for config_file_name in config_file_names {
                println!("{config_file_name}");
            }
        }
        Command::Delete { config_name } => {
            let config_path = if let Some(config_name) = config_name {
                config_path(&config_name)?
            } else {
                config_path_interactive_selection(CONFIG_SELECTION_TO_DELETE_PROMPT_MSG)?
            };
            let delete_confirmation = confirm(CONFIG_DELETE_PROMPT_MSG)?;
            if !delete_confirmation {
                println!("Aborted");
                return Ok(());
            }
            std::fs::remove_file(config_path.clone())?;
            println!("Removed config at: {}", config_path.display());
        }
    };

    Ok(())
}

fn edit_config_by_name_interactively(config_path: &PathBuf) -> eyre::Result<(ZKSyncConfig, bool)> {
    let existing_config: ZKSyncConfig = toml::from_str(&std::fs::read_to_string(config_path)?)?;
    let set_new_config = config_to_edit_is_set(&existing_config)?;
    let new_config = edit_existing_config_interactively(existing_config)?;
    Ok((new_config, set_new_config))
}

fn edit_config_by_name_with_args(
    config_path: &PathBuf,
    opts: EditConfigOpts,
) -> eyre::Result<(ZKSyncConfig, bool)> {
    let existing_config: ZKSyncConfig = toml::from_str(&std::fs::read_to_string(config_path)?)?;
    let set_new_config = config_to_edit_is_set(&existing_config)?;
    let new_config = edit_existing_config_non_interactively(existing_config, opts)?;
    Ok((new_config, set_new_config))
}

fn edit_config_interactively() -> eyre::Result<(ZKSyncConfig, PathBuf, bool)> {
    let config_path = config_path_interactive_selection(CONFIG_EDIT_PROMPT_MSG)?;
    let existing_config: ZKSyncConfig =
        toml::from_str(&std::fs::read_to_string(config_path.clone())?)?;
    let set_new_config = config_to_edit_is_set(&existing_config)?;
    let new_config = edit_existing_config_interactively(existing_config)?;
    Ok((new_config, config_path, set_new_config))
}

fn config_to_edit_is_set(existing_config: &ZKSyncConfig) -> eyre::Result<bool> {
    let selected_config_path = selected_config_path()?;
    if !selected_config_path.exists() {
        return Ok(false);
    }
    let selected_config: ZKSyncConfig =
        toml::from_str(&std::fs::read_to_string(selected_config_path)?)?;
    Ok(&selected_config == existing_config)
}

async fn set_new_config_if_needed(set_new_config: bool, config_path: PathBuf) -> eyre::Result<()> {
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

fn edit_existing_config_interactively(existing_config: ZKSyncConfig) -> eyre::Result<ZKSyncConfig> {
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

fn edit_existing_config_non_interactively(
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
