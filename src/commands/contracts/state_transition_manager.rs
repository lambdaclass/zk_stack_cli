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

fn parse_u256(value: &str) -> Result<U256, String> {
    if value.starts_with("0x") {
        U256::from_str_radix(&value[2..], 16).map_err(|err| err.to_string())
    } else {
        U256::from_dec_str(value).map_err(|err| err.to_string())
    }
}

#[derive(Subcommand)]
pub(crate) enum Command {
    #[command(about = "Get admin address", visible_alias = "a")]
    Admin {
        #[clap(
            short = 's',
            long = "set",
            help = "Propose a new admin",
            exclusive = true
        )]
        new_admin: Option<Address>,
        #[clap(
            short = 'a',
            long = "accept",
            help = "Accept the admin transfer",
            exclusive = true
        )]
        accept: bool,
    },
    #[command(visible_alias = "cfp")]
    ChangeFeeParams {
        #[clap(value_parser = parse_u256)]
        chain_id: U256,
        #[clap(
            help = "The amount of L1 gas required to process the batch (except for the calldata)"
        )]
        batch_overhead_l1_gas: u32,
        #[clap(help = "The maximal number of pubdata that can be emitted per batch")]
        max_pubdata_per_batch: u32,
        max_l2_gas_per_batch: u32,
        #[clap(
            help = "The maximal amount of pubdata a priority transaction is allowed to publish"
        )]
        priority_tx_max_pubdata: u32,
        #[clap(help = "The minimal L2 gas price to be used by L1->L2 transactions")]
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
        #[clap(value_parser = parse_u256)]
        chain_id: U256,
        #[clap(help = "Path to the facetCuts.json file")]
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
        #[clap(value_parser = parse_u256)]
        chain_id: U256,
    },
    #[command(name = "unfreeze", about = "Unfreeze chain", visible_alias = "uf")]
    UnfreezeChain {
        #[clap(value_parser = parse_u256)]
        chain_id: U256,
    },
    #[command(about = "Get all hyperchains addressess", visible_alias = "hc")]
    Hyperchain {
        #[clap(
            long = "id",
            help = "Get address of a hyperchain by its ID",
            exclusive = true,
            value_parser = parse_u256
        )]
        id: Option<U256>,
        #[clap(
            long = "ids",
            help = "Get hyperchain IDs instead of addresses",
            exclusive = true
        )]
        ids: bool,
    },
    #[command(visible_alias = "pv", about = "Get current protocol version")]
    ProtocolVersion {
        #[clap(
            short = 's',
            long = "semantic",
            help = "Print human-readable semantic protocol version"
        )]
        semantic: bool,
    },
    #[command(
        name = "register-deployed-hyperchain",
        about = "Register already deployed hyperchain",
        visible_alias = "rdh"
    )]
    RegisterAlreadyDeployedHyperchain {
        #[clap(value_parser = parse_u256)]
        chain_id: U256,
        hyperchain_address: Address,
    },
    #[command(visible_alias = "ich")]
    SetInitialCutHash {
        #[clap(help = "Path to the facetCuts.json file")]
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
    #[command(visible_alias = "nvu")]
    SetNewVersionUpgrade {
        #[clap(value_parser = parse_u256)]
        old_protocol_version: U256,
        #[clap(value_parser = parse_u256)]
        old_protocol_version_deadline: U256,
        #[clap(value_parser = parse_u256)]
        new_protocol_version: U256,
        #[clap(help = "Path to the facetCuts.json file")]
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
        #[clap(value_parser = parse_u256)]
        chain_id: U256,
        #[clap(value_parser = parse_u256)]
        max_gas_limit: U256,
    },
    #[command(visible_alias = "pa")]
    SetPorterAvailability {
        #[clap(value_parser = parse_u256)]
        chain_id: U256,
        #[clap(help = "0: false, 1: true")]
        is_available: u8,
    },
    #[command(visible_alias = "tm")]
    SetTokenMultiplier {
        #[clap(value_parser = parse_u256)]
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
        #[clap(value_parser = parse_u256)]
        old_protocol_version: U256,
        #[clap(help = "Path to the facetCuts.json file")]
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
        #[clap(value_parser = parse_u256)]
        chain_id: U256,
        #[clap(value_parser = parse_u256)]
        old_protocol_version: U256,
        #[clap(help = "Path to the facetCuts.json file")]
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
    #[command(visible_alias = "vt", about = "Get or set ValidatorTimelock address")]
    ValidatorTimelock {
        #[clap(
            short = 'a',
            long = "address",
            help = "Address to set as ValidatorTimelock"
        )]
        address: Option<Address>,
    },
}

impl Command {
    pub async fn run(self, cfg: ZKSyncConfig) -> eyre::Result<()> {
        let governance = try_governance_from_config(&cfg).await?;
        let state_transition_manager = try_state_transition_manager_from_config(&cfg).await?;
        match self {
            Command::Admin { new_admin, accept } => {
                if !accept && new_admin.is_none() {
                    let admin = state_transition_manager.admin().await?;
                    println!("{:?}", admin);
                } else if accept {
                    state_transition_manager.accept_admin().send().await?;
                } else if let Some(address) = new_admin {
                    let calldata = state_transition_manager
                        .set_pending_admin(address)
                        .function
                        .encode_input(&address.into_tokens())?;
                    run_upgrade(
                        state_transition_manager.address(),
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
            }
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
                    state_transition_manager.address(),
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
                    state_transition_manager.address(),
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
                    state_transition_manager.address(),
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
                    state_transition_manager.address(),
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
            Command::Hyperchain { id, ids } => {
                if ids {
                    let hyperchains = state_transition_manager
                        .get_all_hyperchain_chain_i_ds()
                        .await?;
                    println!("{:?}", hyperchains);
                } else if let Some(chain_id) = id {
                    let address = state_transition_manager.get_hyperchain(chain_id).await?;
                    println!("{chain_id}: {:?}", address);
                } else {
                    let hyperchains = state_transition_manager.get_all_hyperchains().await?;
                    println!("{:?}", hyperchains);
                };
            }
            Command::ProtocolVersion { semantic } => {
                if semantic {
                    let (major, minor, patch) = state_transition_manager
                        .get_semver_protocol_version()
                        .await?;
                    println!("{major}.{minor}.{patch}");
                } else {
                    let version = state_transition_manager.protocol_version().await?;
                    println!("{}", version);
                }
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
                    state_transition_manager.address(),
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
                    state_transition_manager.address(),
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
                    state_transition_manager.address(),
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
                    state_transition_manager.address(),
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
                    state_transition_manager.address(),
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
                    state_transition_manager.address(),
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
                    state_transition_manager.address(),
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
                    state_transition_manager.address(),
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
            Command::ValidatorTimelock { address } => {
                if let Some(address) = address {
                    let calldata = state_transition_manager
                        .set_validator_timelock(address)
                        .function
                        .encode_input(&address.into_tokens())?;
                    run_upgrade(
                        state_transition_manager.address(),
                        calldata.into(),
                        false,
                        true,
                        0.into(),
                        false,
                        governance,
                        cfg,
                    )
                    .await?;
                } else {
                    let address = state_transition_manager.validator_timelock().await?;
                    println!("{address}");
                }
            }
        };
        Ok(())
    }
}
