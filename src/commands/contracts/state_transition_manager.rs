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
    #[command(name = "freeze", about = "Freeze chain", visible_alias = "fr")]
    FreezeChain {
        #[clap(index = 1, required = true)]
        chain_id: U256,
    },
    #[command(name = "unfreeze", about = "Unfreeze chain", visible_alias = "uf")]
    UnfreezeChain {
        #[clap(index = 1, required = true)]
        chain_id: U256,
    },
    #[command(
        name = "register-deployed-hyperchain",
        about = "Register already deployed hyperchain",
        visible_alias = "rdh"
    )]
    RegisterAlreadyDeployedHyperchain {
        #[clap(index = 1, required = true)]
        chain_id: U256,
        #[clap(index = 2, required = true)]
        hyperchain_address: Address,
    },
    #[command(
        name = "priority-gas-limit",
        about = "Set priority tx max gas limit",
        visible_alias = "pgl"
    )]
    SetPriorityTxMaxGasLimit {
        #[clap(index = 1, required = true)]
        chain_id: U256,
        #[clap(index = 2, required = true)]
        max_gas_limit: U256,
    },
}

impl Command {
    pub async fn run(self, cfg: ZKSyncConfig) -> eyre::Result<()> {
        let governance = try_governance_from_config(&cfg).await?;
        let state_transition_manager = try_state_transition_manager_from_config(&cfg).await?;
        match self {
            Command::FreezeChain { chain_id } => {
                let calldata = state_transition_manager
                    .freeze_chain(chain_id)
                    .function
                    .encode_input(&chain_id.into_tokens())?;
                run_upgrade(
                    calldata.into(),
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
                let calldata = state_transition_manager
                    .unfreeze_chain(chain_id)
                    .function
                    .encode_input(&chain_id.into_tokens())?;
                run_upgrade(
                    calldata.into(),
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
                let calldata = state_transition_manager
                    .register_already_deployed_hyperchain(chain_id, hyperchain_address)
                    .function
                    .encode_input(
                        &[chain_id.into_tokens(), hyperchain_address.into_tokens()].concat(),
                    )?;
                run_upgrade(
                    calldata.into(),
                    false,
                    true,
                    0.into(),
                    false,
                    governance,
                    cfg,
                )
                .await?;
            }
            Command::SetPriorityTxMaxGasLimit {
                chain_id,
                max_gas_limit,
            } => {
                let calldata = state_transition_manager
                    .set_priority_tx_max_gas_limit(chain_id, max_gas_limit)
                    .function
                    .encode_input(
                        &[chain_id.into_tokens(), max_gas_limit.into_tokens()].concat(),
                    )?;
                run_upgrade(
                    calldata.into(),
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
