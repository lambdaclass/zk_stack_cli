use std::{fs::File, io};

use crate::{
    config::ZKSyncConfig,
    utils::contracts::{try_governance_from_config, try_state_transition_manager_from_config},
};
use clap::Subcommand;
use serde::Deserialize;
use zksync_ethers_rs::{
    abi::Tokenize,
    types::{Address, Selector},
};
use zksync_ethers_rs::{
    contracts::state_transition_manager::{DiamondCutData, FacetCut, FeeParams},
    types::U256,
};

use super::governance::run_upgrade;

#[derive(Deserialize)]
#[repr(u8)]
enum PubdataPricingMode {
    Rollup = 0,
    Validium,
}

#[derive(Deserialize)]
#[repr(u8)]
enum FacetCutType {
    Add = 0,
    Replace,
    Remove,
}

#[derive(Deserialize)]
struct FacetCutDef {
    facet: Address,
    action: FacetCutType,
    selectors: Vec<Selector>,
    is_freezable: bool,
}

impl From<FacetCutDef> for FacetCut {
    fn from(value: FacetCutDef) -> Self {
        let selectors = value.selectors;
        FacetCut {
            facet: value.facet,
            action: value.action as u8,
            selectors,
            is_freezable: value.is_freezable,
        }
    }
}

fn diamond_cut_data_from_params(
    facet_cuts_path: String,
    init_address: Option<Address>,
    init_calldata: Option<Vec<u8>>,
) -> io::Result<DiamondCutData> {
    let facet_cuts: Vec<FacetCutDef> = serde_json::from_reader(File::open(facet_cuts_path)?)?;
    Ok(DiamondCutData {
        facet_cuts: facet_cuts.into_iter().map(FacetCut::from).collect(),
        init_address: init_address.unwrap_or(Address::zero()),
        init_calldata: init_calldata.unwrap_or(Vec::new()).into(),
    })
}

#[derive(Subcommand)]
pub(crate) enum Command {
    #[command(visible_alias = "cfp")]
    ChangeFeeParams {
        #[clap(required = true)]
        chain_id: U256,
        #[clap(required = true)]
        batch_overhead_l1_gas: u32,
        #[clap(required = true)]
        max_pubdata_per_batch: u32,
        #[clap(required = true)]
        max_l2_gas_per_batch: u32,
        #[clap(required = true)]
        priority_tx_max_pubdata: u32,
        #[clap(required = true)]
        minimal_l2_gas_price: u64,
        #[clap(
            short = 'r',
            long = "rollup-mode",
            required = false,
            default_value = "0",
            help = "Default"
        )]
        rollup_mode: bool,
        #[clap(
            short = 'v',
            long = "validium-mode",
            required = false,
            default_value = "0"
        )]
        validium_mode: bool,
    },
    #[command(visible_alias = "eu")]
    ExecuteUpgrade {
        #[clap(required = true)]
        chain_id: U256,
        #[clap(required = true, help = "Path to the facetCuts.json file")]
        facet_cuts_path: String,
        #[clap(
            name = "init-address",
            short = 'a',
            required = false,
            requires = "init-calldata",
            help = "The address that's delegate called after setting up new facet changes"
        )]
        init_address: Option<Address>,
        #[clap(
            name = "init-calldata",
            short = 'c',
            required = false,
            requires = "init-address",
            help = "Calldata for the delegate call to initAddress"
        )]
        init_calldata: Option<Vec<u8>>,
    },
    #[command(name = "freeze", about = "Freeze chain", visible_alias = "fr")]
    FreezeChain {
        #[clap(required = true)]
        chain_id: U256,
    },
    #[command(name = "unfreeze", about = "Unfreeze chain", visible_alias = "uf")]
    UnfreezeChain {
        #[clap(required = true)]
        chain_id: U256,
    },
    #[command(
        name = "register-deployed-hyperchain",
        about = "Register already deployed hyperchain",
        visible_alias = "rdh"
    )]
    RegisterAlreadyDeployedHyperchain {
        #[clap(required = true)]
        chain_id: U256,
        #[clap(required = true)]
        hyperchain_address: Address,
    },
    #[command(name = "set-initial-cut-hash", visible_alias = "ich")]
    SetInitialCutHash {
        #[clap(required = true, help = "Path to the facetCuts.json file")]
        facet_cuts_path: String,
        #[clap(
            name = "init-address",
            short = 'a',
            required = false,
            requires = "init-calldata",
            help = "The address that's delegate called after setting up new facet changes"
        )]
        init_address: Option<Address>,
        #[clap(
            name = "init-calldata",
            short = 'c',
            required = false,
            requires = "init-address",
            help = "Calldata for the delegate call to initAddress"
        )]
        init_calldata: Option<Vec<u8>>,
    },
    #[command(name = "set-new-version-upgrade", visible_alias = "nvu")]
    SetNewVersionUpgrade {
        #[clap(required = true)]
        old_protocol_version: U256,
        #[clap(required = true)]
        old_protocol_version_deadline: U256,
        #[clap(required = true)]
        new_protocol_version: U256,
        #[clap(required = true, help = "Path to the facetCuts.json file")]
        facet_cuts_path: String,
        #[clap(
            name = "init-address",
            short = 'a',
            required = false,
            requires = "init-calldata",
            help = "The address that's delegate called after setting up new facet changes"
        )]
        init_address: Option<Address>,
        #[clap(
            name = "init-calldata",
            short = 'c',
            required = false,
            requires = "init-address",
            help = "Calldata for the delegate call to initAddress"
        )]
        init_calldata: Option<Vec<u8>>,
    },
    #[command(
        name = "set-priority-gas-limit",
        about = "Set priority tx max gas limit",
        visible_alias = "pgl"
    )]
    SetPriorityTxMaxGasLimit {
        #[clap(required = true)]
        chain_id: U256,
        #[clap(required = true)]
        max_gas_limit: U256,
    },
    #[command(visible_alias = "pa")]
    SetPorterAvailability {
        #[clap(required = true)]
        chain_id: U256,
        #[clap(required = true, help = "0: false, 1: true")]
        is_available: u8,
    },
    #[command(visible_alias = "tm")]
    SetTokenMultiplier {
        #[clap(required = true)]
        chain_id: U256,
        #[clap(short = 'n', long = "nominator", required = false, default_value = "1")]
        nominator: u128,
        #[clap(
            short = 'd',
            long = "denominator",
            required = false,
            default_value = "1"
        )]
        denominator: u128,
    },
    #[command(visible_alias = "udc")]
    SetUpgradeDiamondCut {
        #[clap(required = true)]
        old_protocol_version: U256,
        #[clap(required = true, help = "Path to the facetCuts.json file")]
        facet_cuts_path: String,
        #[clap(
            name = "init-address",
            short = 'a',
            required = false,
            requires = "init-calldata",
            help = "The address that's delegate called after setting up new facet changes"
        )]
        init_address: Option<Address>,
        #[clap(
            name = "init-calldata",
            short = 'c',
            required = false,
            requires = "init-address",
            help = "Calldata for the delegate call to initAddress"
        )]
        init_calldata: Option<Vec<u8>>,
    },
    #[command(visible_alias = "uc")]
    UpgradeChainFromVersion {
        #[clap(required = true)]
        chain_id: U256,
        #[clap(required = true)]
        old_protocol_version: U256,
        #[clap(required = true, help = "Path to the facetCuts.json file")]
        facet_cuts_path: String,
        #[clap(
            name = "init-address",
            short = 'a',
            required = false,
            requires = "init-calldata",
            help = "The address that's delegate called after setting up new facet changes"
        )]
        init_address: Option<Address>,
        #[clap(
            name = "init-calldata",
            short = 'c',
            required = false,
            requires = "init-address",
            help = "Calldata for the delegate call to initAddress"
        )]
        init_calldata: Option<Vec<u8>>,
    },
}

impl Command {
    pub async fn run(self, cfg: ZKSyncConfig) -> eyre::Result<()> {
        let governance = try_governance_from_config(&cfg).await?;
        let state_transition_manager = try_state_transition_manager_from_config(&cfg).await?;
        match self {
            Command::ChangeFeeParams {
                chain_id,
                batch_overhead_l1_gas,
                max_pubdata_per_batch,
                max_l2_gas_per_batch,
                priority_tx_max_pubdata,
                minimal_l2_gas_price,
                rollup_mode,
                validium_mode,
            } => {
                let fee_params = FeeParams {
                    pubdata_pricing_mode: match (rollup_mode, validium_mode) {
                        (false, true) => PubdataPricingMode::Validium as u8,
                        _ => PubdataPricingMode::Rollup as u8,
                    },
                    batch_overhead_l1_gas,
                    max_pubdata_per_batch,
                    max_l2_gas_per_batch,
                    priority_tx_max_pubdata,
                    minimal_l2_gas_price,
                };
                let calldata = state_transition_manager
                    .change_fee_params(chain_id, fee_params.clone())
                    .function
                    .encode_input(&[chain_id.into_tokens(), fee_params.into_tokens()].concat())?;
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
            Command::ExecuteUpgrade {
                chain_id,
                facet_cuts_path,
                init_address,
                init_calldata,
            } => {
                let diamond_cut_data =
                    diamond_cut_data_from_params(facet_cuts_path, init_address, init_calldata)?;
                let calldata = state_transition_manager
                    .execute_upgrade(chain_id, diamond_cut_data.clone())
                    .function
                    .encode_input(
                        &[chain_id.into_tokens(), diamond_cut_data.into_tokens()].concat(),
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
            Command::SetInitialCutHash {
                facet_cuts_path,
                init_address,
                init_calldata,
            } => {
                let diamond_cut_data =
                    diamond_cut_data_from_params(facet_cuts_path, init_address, init_calldata)?;
                let calldata = state_transition_manager
                    .set_initial_cut_hash(diamond_cut_data.clone())
                    .function
                    .encode_input(&diamond_cut_data.into_tokens())?;
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
            Command::SetPorterAvailability {
                chain_id,
                is_available,
            } => {
                let is_available: bool = is_available != 0;
                let calldata = state_transition_manager
                    .set_porter_availability(chain_id, is_available)
                    .function
                    .encode_input(&[chain_id.into_tokens(), is_available.into_tokens()].concat())?;
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
            Command::SetTokenMultiplier {
                chain_id,
                nominator,
                denominator,
            } => {
                let calldata = state_transition_manager
                    .set_token_multiplier(chain_id, nominator, denominator)
                    .function
                    .encode_input(
                        &[
                            chain_id.into_tokens(),
                            nominator.into_tokens(),
                            denominator.into_tokens(),
                        ]
                        .concat(),
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
            Command::SetUpgradeDiamondCut {
                old_protocol_version,
                facet_cuts_path,
                init_address,
                init_calldata,
            } => {
                let diamond_cut_data =
                    diamond_cut_data_from_params(facet_cuts_path, init_address, init_calldata)?;
                let calldata = state_transition_manager
                    .set_upgrade_diamond_cut(diamond_cut_data.clone(), old_protocol_version)
                    .function
                    .encode_input(
                        &[
                            diamond_cut_data.into_tokens(),
                            old_protocol_version.into_tokens(),
                        ]
                        .concat(),
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
            Command::UpgradeChainFromVersion {
                chain_id,
                old_protocol_version,
                facet_cuts_path,
                init_address,
                init_calldata,
            } => {
                let diamond_cut_data =
                    diamond_cut_data_from_params(facet_cuts_path, init_address, init_calldata)?;
                let calldata = state_transition_manager
                    .upgrade_chain_from_version(
                        chain_id,
                        old_protocol_version,
                        diamond_cut_data.clone(),
                    )
                    .function
                    .encode_input(
                        &[
                            chain_id.into_tokens(),
                            old_protocol_version.into_tokens(),
                            diamond_cut_data.into_tokens(),
                        ]
                        .concat(),
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
            Command::SetNewVersionUpgrade {
                old_protocol_version,
                old_protocol_version_deadline,
                new_protocol_version,
                facet_cuts_path,
                init_address,
                init_calldata,
            } => {
                let diamond_cut_data =
                    diamond_cut_data_from_params(facet_cuts_path, init_address, init_calldata)?;
                let calldata = state_transition_manager
                    .set_new_version_upgrade(
                        diamond_cut_data.clone(),
                        old_protocol_version,
                        old_protocol_version_deadline,
                        new_protocol_version,
                    )
                    .function
                    .encode_input(
                        &[
                            diamond_cut_data.into_tokens(),
                            old_protocol_version.into_tokens(),
                            old_protocol_version_deadline.into_tokens(),
                            new_protocol_version.into_tokens(),
                        ]
                        .concat(),
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
