use std::sync::Arc;

use crate::{
    config::ZKSyncConfig,
    utils::{contracts::try_bridgehub_from_config, try_l1_signer_from_config},
};
use clap::Subcommand;
use eyre::ContextCompat;
use zksync_ethers_rs::{
    contracts::bridgehub::Bridgehub,
    signers::LocalWallet,
    types::{Address, U256},
};

#[derive(Subcommand)]
pub(crate) enum Command {
    #[clap(
        about = "Get the StateTransitionManager contract address of a chain.",
        visible_alias = "stm"
    )]
    StateTransitionManager {
        #[clap(long, value_parser = U256::from_dec_str)]
        chain_id: U256,
    },
    #[clap(
        about = "Get the base token contract of a chain.",
        visible_alias = "bt"
    )]
    BaseToken {
        #[clap(long, value_parser = U256::from_dec_str)]
        chain_id: U256,
    },
    #[clap(about = "Get the bridge contract admin address.")]
    Admin,
    #[clap(
        about = "Set a new admin of the Bridgehub. Only the Bridgehub owner or the current admin can do this.",
        visible_alias = "spa"
    )]
    SetPendingAdmin { new_pending_admin: Address },
    #[clap(
        about = "Accept the admin of the Bridgehub. Only the pending admin can do this.",
        visible_alias = "aa"
    )]
    AcceptAdmin {
        pending_admin_private_key: LocalWallet,
    },
    #[clap(
        about = "Get the Hyperchain contract address of a chain.",
        visible_aliases = ["h", "hyperchain"]
    )]
    GetHyperchain {
        #[clap(long, value_parser = U256::from_dec_str)]
        chain_id: U256,
    },
}

impl Command {
    pub async fn run(self, cfg: ZKSyncConfig) -> eyre::Result<()> {
        let bridgehub = try_bridgehub_from_config(&cfg).await?;
        match self {
            Command::StateTransitionManager { chain_id } => {
                let state_transition_manager: Address =
                    bridgehub.state_transition_manager(chain_id).call().await?;
                println!("STM for chain ID {chain_id:?}: {state_transition_manager:?}",);
            }
            Command::BaseToken { chain_id } => {
                let base_token: Address = bridgehub.base_token(chain_id).call().await?;
                println!("Base token for chain ID {chain_id:?}: {base_token:?}",);
            }
            Command::Admin => {
                let bridgehub_admin: Address = bridgehub.admin().call().await?;
                if bridgehub_admin == Address::default() {
                    println!("Bridgehub admin is not set");
                } else {
                    println!("Bridgehub admin: {bridgehub_admin:?}");
                }
            }
            Command::SetPendingAdmin { new_pending_admin } => {
                // TODO: Do not repeat the same code for calling the set_pending_admin method.
                let transaction_receipt = if let Some(ref admin_private_key) =
                    cfg.bridgehub.admin_private_key
                {
                    let current_admin =
                        try_l1_signer_from_config(admin_private_key.parse()?, &cfg).await?;
                    // We need to instantiate a Bridgehub with the pending admin as the signer
                    // to be able to call accept_admin
                    let bridgehub = Bridgehub::new(bridgehub.address(), Arc::new(current_admin));
                    bridgehub
                        .set_pending_admin(new_pending_admin)
                        .send()
                        .await?
                        .await?
                        .context("No transaction receipt for bridgehub admin acceptance")?
                } else {
                    bridgehub
                        .set_pending_admin(new_pending_admin)
                        .send()
                        .await?
                        .await?
                        .context("No transaction receipt for bridgehub admin acceptance")?
                };
                println!(
                    "New Bridgehub pending admin: {:?}",
                    transaction_receipt.transaction_hash
                );
            }
            Command::AcceptAdmin {
                pending_admin_private_key,
            } => {
                let pending_admin =
                    try_l1_signer_from_config(pending_admin_private_key, &cfg).await?;
                // We need to instantiate a Bridgehub with the pending admin as the signer
                // to be able to call accept_admin
                let bridgehub = Bridgehub::new(bridgehub.address(), Arc::new(pending_admin));
                let transaction_receipt = bridgehub
                    .accept_admin()
                    .send()
                    .await?
                    .await?
                    .context("No transaction receipt for bridgehub admin acceptance")?;
                println!(
                    "New Bridgehub admin accepted: {:?}",
                    transaction_receipt.transaction_hash
                );
            }
            Command::GetHyperchain { chain_id } => {
                let hyperchain: Address = bridgehub.get_hyperchain(chain_id).call().await?;
                println!("Hyperchain address for chain ID {chain_id:?}: {hyperchain:?}");
            }
        };
        Ok(())
    }
}
