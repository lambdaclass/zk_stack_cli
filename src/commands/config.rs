use crate::{
    config::{
        BridgehubConfig, Database, DatabaseConfig, GovernanceConfig, NetworkConfig, WalletConfig,
        ZKSyncConfig,
    },
    utils::{
        config::{
            config_file_names, config_path, config_path_interactive_selection, confirm,
            confirm_config_creation, edit_config_by_name_interactively,
            edit_config_by_name_with_args, edit_config_interactively, prompt_zksync_config,
            selected_config_path, set_new_config,
        },
        messages::{
            CONFIG_DELETE_PROMPT_MSG, CONFIG_OVERRIDE_PROMPT_MSG,
            CONFIG_SELECTION_TO_DELETE_PROMPT_MSG, CONFIG_SET_PROMPT_MSG,
            CONFIG_TO_DISPLAY_PROMPT_MSG,
        },
    },
};
use clap::{Parser, Subcommand};
use eyre::Context;
use std::env;
use std::str::FromStr;
use zksync_ethers_rs::types::{zksync::ETHEREUM_ADDRESS, Address, H160};

#[derive(Subcommand)]
pub(crate) enum Command {
    #[clap(about = "Edit an existing config.")]
    Edit {
        config_name: Option<String>,
        #[command(flatten)]
        opts: EditConfigOpts,
    },
    #[clap(about = "Create a new config.")]
    Create { config_name: String },
    #[clap(about = "Create a new config from zksync's ENV file variables.")]
    CreateFromEnv {
        config_name: String,
        config_override: bool,
    },
    #[clap(about = "Set the config to use.")]
    Set { config_name: Option<String> },
    #[clap(about = "Display a config.")]
    Display { config_name: Option<String> },
    #[clap(about = "List all configs.")]
    List,
    #[clap(about = "Delete a config.")]
    Delete { config_name: Option<String> },
}

#[derive(Parser)]
pub struct EditConfigOpts {
    #[arg(long, requires = "config_name", required = false)]
    pub l1_rpc_url: Option<String>,
    #[arg(long, requires = "config_name", required = false)]
    pub l1_chain_id: Option<u64>,
    #[arg(long, requires = "config_name", required = false)]
    pub l2_rpc_url: Option<String>,
    #[arg(long, requires = "config_name", required = false)]
    pub l2_chain_id: Option<u64>,
    #[arg(long, requires = "config_name", required = false)]
    pub l2_explorer_url: Option<String>,
    #[arg(long, requires = "config_name", required = false)]
    pub l1_explorer_url: Option<String>,
    #[arg(long, requires = "config_name", required = false)]
    pub private_key: Option<String>,
    #[arg(long, requires = "config_name", required = false)]
    pub address: Option<Address>,
    #[arg(long, requires = "config_name", required = false)]
    pub governance: Option<Address>,
    #[arg(long, requires = "config_name", required = false)]
    pub governance_owner: Option<String>,
    #[arg(long, requires = "config_name", required = false)]
    pub bridgehub_admin: Option<String>,
    #[arg(long, requires = "config_name", required = false)]
    pub bridgehub_owner: Option<String>,
    #[arg(long, requires = "config_name", required = false)]
    pub server_db_url: Option<Database>,
    #[arg(long, requires = "config_name", required = false)]
    pub prover_db_url: Option<Database>,
}

impl EditConfigOpts {
    pub fn is_empty(&self) -> bool {
        self.l1_explorer_url.is_none()
            && self.l1_rpc_url.is_none()
            && self.l2_explorer_url.is_none()
            && self.l2_rpc_url.is_none()
            && self.private_key.is_none()
            && self.address.is_none()
            && self.governance.is_none()
            && self.governance_owner.is_none()
            && self.bridgehub_admin.is_none()
            && self.bridgehub_owner.is_none()
            && self.server_db_url.is_none()
            && self.prover_db_url.is_none()
    }
}

impl Command {
    pub async fn run(self) -> eyre::Result<()> {
        match self {
            Command::Edit { config_name, opts } => {
                let (new_config, config_path) = if let Some(ref config_name) = config_name {
                    let config_path = config_path(config_name)?;
                    if !config_path.exists() {
                        return confirm_config_creation(config_name.clone()).await;
                    }
                    let new_config = if opts.is_empty() {
                        edit_config_by_name_interactively(&config_path)?
                    } else {
                        edit_config_by_name_with_args(&config_path, opts)?
                    };
                    (new_config, config_path)
                } else {
                    edit_config_interactively()?
                };
                let toml_config = toml::to_string_pretty(&new_config)?;
                std::fs::write(&config_path, &toml_config)?;
                set_new_config(config_path.clone()).await?;
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
            // This command makes use of the zksync-era's env file
            // Extra env variables needed:
            // L1_EXPLORER_URL
            // L2_EXPLORER_URL
            // WALLET_ADDR
            // WALLET_PK
            // GOVERNANCE_ADDRESS
            // GOVERNANCE_OWNER_PK
            // BRIDGEHUB_OWNER_PK
            // BRIDGEHUB_ADMIN_PK
            Command::CreateFromEnv {
                config_name,
                config_override,
            } => {
                let config_path = config_path(&config_name)?;
                if config_path.exists() && !config_override {
                    println!("Aborted");
                    return Ok::<(), eyre::Error>(());
                }
                let config = ZKSyncConfig {
                    network: NetworkConfig {
                        l1_rpc_url: Some(
                            env::var("ETH_CLIENT_WEB3_URL")
                                .context("ETH_CLIENT_WEB3_URL Not present")?,
                        ),
                        l1_chain_id: Some(
                            env::var("ETH_CLIENT_CHAIN_ID")
                                .context("ETH_CLIENT_CHAIN_ID Not present")?
                                .parse::<u64>()?,
                        ),
                        l1_explorer_url: Some(
                            env::var("L1_EXPLORER_URL").context("L1_EXPLORER_URL Not present")?,
                        ),
                        l2_rpc_url: env::var("API_WEB3_JSON_RPC_HTTP_URL")
                            .context("API_WEB3_JSON_RPC_HTTP_URL Not present")?,
                        l2_chain_id: Some(
                            env::var("CHAIN_ETH_ZKSYNC_NETWORK_ID")
                                .context("CHAIN_ETH_ZKSYNC_NETWORK_ID Not present")?
                                .parse::<u64>()?,
                        ),
                        l2_explorer_url: Some(
                            env::var("L2_EXPLORER_URL").context("L2_EXPLORER_URL Not present")?,
                        ),
                    },
                    wallet: Some(WalletConfig {
                        address: H160::from_str(
                            &env::var("WALLET_ADDR").context("WALLET_ADDR Not present")?,
                        )?,
                        private_key: env::var("WALLET_PK").context("WALLET_PK Not present")?,
                    }),
                    db: Some(DatabaseConfig {
                        server: Database::from_str(
                            &env::var("DATABASE_URL").context("DATABASE_URL Not present")?,
                        )?,
                        prover: Database::from_str(
                            &env::var("DATABASE_PROVER_URL")
                                .context("DATABASE_PROVER_URL Not present")?,
                        )?,
                    }),
                    governance: GovernanceConfig {
                        address: H160::from_str(
                            &env::var("GOVERNANCE_ADDRESS")
                                .context("GOVERNANCE_ADDRESS Not present")?,
                        )?,
                        owner_private_key: env::var("GOVERNANCE_OWNER_PK")
                            .context("GOVERNANCE_OWNER_PK Not present")?,
                    },
                    bridgehub: BridgehubConfig {
                        admin_private_key: Some(
                            env::var("BRIDGEHUB_ADMIN_PK")
                                .context("BRIDGEHUB_ADMIN_PK Not present")?,
                        ),
                        owner_private_key: Some(
                            env::var("BRIDGEHUB_OWNER_PK")
                                .context("BRIDGEHUB_OWNER_PK Not present")?,
                        ),
                    },
                };
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
}
