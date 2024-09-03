use crate::{
    config::ZKSyncConfig,
    utils::contracts::{try_governance_from_config, try_state_transition_manager_from_config},
};
use clap::Subcommand;
use zksync_ethers_rs::types::U256;
use zksync_ethers_rs::{abi::Tokenize, types::Address};

use super::governance::run_upgrade;

#[derive(Subcommand)]
pub(crate) enum Command {
    FreezeChain {
        #[clap(index = 1, required = true)]
        chain_id: U256,
    },
    UnfreezeChain {
        #[clap(index = 1, required = true)]
        chain_id: U256,
    },
    RegisterAlreadyDeployedHyperchain {
        #[clap(index = 1, required = true)]
        chain_id: U256,
        #[clap(index = 2, required = true)]
        hyperchain_address: Address,
    },
}

impl Command {
    pub async fn run(self, cfg: ZKSyncConfig) -> eyre::Result<()> {
        let governance = try_governance_from_config(&cfg).await?;
        let state_transition_manager = try_state_transition_manager_from_config(&cfg).await?;
        match self {
            Command::FreezeChain { chain_id } => {
                let freeze_calldata = state_transition_manager
                    .freeze_chain(chain_id)
                    .function
                    .encode_input(&chain_id.into_tokens())?;
                run_upgrade(
                    freeze_calldata.into(),
                    false,
                    true,
                    0.into(),
                    false,
                    governance,
                    cfg,
                )
                .await?;
            }
            Command::UnfreezeChain { chain_id } => {
                let unfreeze_calldata = state_transition_manager
                    .unfreeze_chain(chain_id)
                    .function
                    .encode_input(&chain_id.into_tokens())?;
                run_upgrade(
                    unfreeze_calldata.into(),
                    false,
                    true,
                    0.into(),
                    false,
                    governance,
                    cfg,
                )
                .await?;
            }
            Command::RegisterAlreadyDeployedHyperchain {
                chain_id,
                hyperchain_address,
            } => {
                let register_hyperchain_calldata = state_transition_manager
                    .register_already_deployed_hyperchain(chain_id, hyperchain_address)
                    .function
                    .encode_input(
                        &[chain_id.into_tokens(), hyperchain_address.into_tokens()].concat(),
                    )?;
                run_upgrade(
                    register_hyperchain_calldata.into(),
                    false,
                    true,
                    0.into(),
                    false,
                    governance,
                    cfg,
                )
                .await?;
            }
        };
        Ok(())
    }
}
