use crate::{
    commands::config::common::{
        config_path, config_path_interactive_selection, confirm_config_creation, prompt,
        ADDRESS_PROMPT_MSG, CONFIG_EDIT_PROMPT_MSG, DEFAULT_ADDRESS, DEFAULT_L1_EXPLORER_URL,
        DEFAULT_L1_RPC_URL, DEFAULT_L2_EXPLORER_URL, DEFAULT_PRIVATE_KEY,
        L1_EXPLORER_URL_PROMPT_MSG, L1_RPC_URL_PROMPT_MSG, L2_EXPLORER_URL_PROMPT_MSG,
        L2_RPC_URL_PROMPT_MSG, PRIVATE_KEY_PROMPT_MSG,
    },
    config::{NetworkConfig, WalletConfig, ZKSyncConfig},
};
use clap::Args as ClapArgs;
use zksync_ethers_rs::types::Address;

#[derive(ClapArgs, PartialEq)]
pub(crate) struct Args {
    #[clap(long = "name", required_unless_present = "edit_interactively")]
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
    #[clap(short, long = "interactively", required = false, conflicts_with_all = &["l1_rpc_url", "l2_rpc_url", "l2_explorer_url", "l1_explorer_url", "private_key", "address"], required_unless_present = "config_name")]
    pub edit_interactively: bool,
}

pub(crate) async fn run(args: Args) -> eyre::Result<()> {
    let (new_config, config_path) = match (&args.config_name, &args.edit_interactively) {
        (None, true) => {
            let config_path = config_path_interactive_selection(CONFIG_EDIT_PROMPT_MSG)?;
            let existing_config: ZKSyncConfig =
                toml::from_str(&std::fs::read_to_string(config_path.clone())?)?;
            let new_config = edit_existing_config_interactively(existing_config)?;

            (new_config, config_path)
        }
        (Some(config_name), true) => {
            let config_path = config_path(config_name)?;
            if !config_path.exists() {
                return confirm_config_creation(config_name.clone()).await;
            }
            let existing_config: ZKSyncConfig =
                toml::from_str(&std::fs::read_to_string(config_path.clone())?)?;
            let new_config = edit_existing_config_interactively(existing_config)?;

            (new_config, config_path)
        }
        (Some(config_name), false) => {
            let config_path = config_path(config_name)?;
            if !config_path.exists() {
                return confirm_config_creation(config_name.clone()).await;
            }
            let existing_config: ZKSyncConfig =
                toml::from_str(&std::fs::read_to_string(config_path.clone())?)?;
            let new_config = edit_existing_config_non_interactively(existing_config, args)?;

            (new_config, config_path)
        }
        _ => unreachable!(),
    };

    let toml_config = toml::to_string_pretty(&new_config)?;
    std::fs::write(&config_path, &toml_config)?;
    println!("Config updated at: {}", config_path.display());
    println!("\n{toml_config}");
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
            l2_rpc_url: prompt(
                L2_RPC_URL_PROMPT_MSG,
                existing_config.network.l2_rpc_url.into(),
            )?,
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
        wallet: existing_config.wallet.and_then(|existing_wallet_config| {
            Some(WalletConfig {
                private_key: args
                    .private_key
                    .unwrap_or(existing_wallet_config.private_key),
                address: args.address.unwrap_or(existing_wallet_config.address),
            })
        }),
    };
    Ok(config)
}
