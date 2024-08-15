use std::path::PathBuf;

use crate::{
    commands::config::{
        common::{
            config_path, config_path_interactive_selection, confirm_config_creation, prompt,
            selected_config_path, ADDRESS_PROMPT_MSG, CONFIG_EDIT_PROMPT_MSG, DEFAULT_ADDRESS,
            DEFAULT_L1_EXPLORER_URL, DEFAULT_L1_RPC_URL, DEFAULT_L2_EXPLORER_URL,
            DEFAULT_PRIVATE_KEY, L1_EXPLORER_URL_PROMPT_MSG, L1_RPC_URL_PROMPT_MSG,
            L2_EXPLORER_URL_PROMPT_MSG, L2_RPC_URL_PROMPT_MSG, PRIVATE_KEY_PROMPT_MSG,
        },
        edit, set,
    },
    config::{NetworkConfig, WalletConfig, ZKSyncConfig},
};
use clap::Args as ClapArgs;
use eyre::ContextCompat;
use zksync_ethers_rs::types::Address;

#[derive(ClapArgs, PartialEq)]
pub(crate) struct Args {
    #[clap(long = "name")]
    pub config_name: Option<String>,
    #[clap(
        long,
        conflicts_with = "interactively",
        requires = "config_name",
        required = false
    )]
    pub l1_rpc_url: Option<String>,
    #[clap(
        long,
        conflicts_with = "interactively",
        requires = "config_name",
        required = false
    )]
    pub l2_rpc_url: Option<String>,
    #[clap(
        long,
        conflicts_with = "interactively",
        requires = "config_name",
        required = false
    )]
    pub l2_explorer_url: Option<String>,
    #[clap(
        long,
        conflicts_with = "interactively",
        requires = "config_name",
        required = false
    )]
    pub l1_explorer_url: Option<String>,
    #[clap(
        long,
        conflicts_with = "interactively",
        requires = "config_name",
        required = false
    )]
    pub private_key: Option<String>,
    #[clap(
        long,
        conflicts_with = "interactively",
        requires = "config_name",
        required = false
    )]
    pub address: Option<Address>,
}

pub(crate) async fn run(args: Args) -> eyre::Result<()> {
    let (new_config, config_path, set_new_config) = if let Some(ref config_name) = args.config_name
    {
        let config_path = config_path(config_name)?;
        if !config_path.exists() {
            return confirm_config_creation(config_name.clone()).await;
        }
        let (new_config, set_new_config) = if args.l1_explorer_url.is_none()
            && args.l1_rpc_url.is_none()
            && args.l2_explorer_url.is_none()
            && args.l2_rpc_url.is_none()
            && args.private_key.is_none()
            && args.address.is_none()
        {
            edit_config_by_name_interactively(&config_path)?
        } else {
            edit_config_by_name_with_args(&config_path, args)?
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
    args: Args,
) -> eyre::Result<(ZKSyncConfig, bool)> {
    let existing_config: ZKSyncConfig = toml::from_str(&std::fs::read_to_string(config_path)?)?;
    let set_new_config = config_to_edit_is_set(&existing_config)?;
    let new_config = edit_existing_config_non_interactively(existing_config, args)?;
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
        set::run(set::Args {
            config_name: Some(
                config_path
                    .file_stem()
                    .context("There's no file name")?
                    .to_os_string()
                    .into_string()
                    .map_err(|_| eyre::eyre!("Invalid file name"))?,
            ),
        })
        .await?;
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
    };
    Ok(config)
}

fn edit_existing_config_non_interactively(
    existing_config: ZKSyncConfig,
    args: Args,
) -> eyre::Result<ZKSyncConfig> {
    let config = ZKSyncConfig {
        network: NetworkConfig {
            l1_rpc_url: args.l1_rpc_url.or(existing_config.network.l1_rpc_url),
            l2_rpc_url: args
                .l2_rpc_url
                .unwrap_or(existing_config.network.l2_rpc_url),
            l2_explorer_url: args
                .l2_explorer_url
                .or(existing_config.network.l2_explorer_url),
            l1_explorer_url: args
                .l1_explorer_url
                .or(existing_config.network.l1_explorer_url),
        },
        wallet: existing_config
            .wallet
            .map(|existing_wallet_config| WalletConfig {
                private_key: args
                    .private_key
                    .unwrap_or(existing_wallet_config.private_key),
                address: args.address.unwrap_or(existing_wallet_config.address),
            }),
    };
    Ok(config)
}
