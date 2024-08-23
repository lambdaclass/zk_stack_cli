use crate::{config::ZKSyncConfig, utils::contracts::try_governance_from_config};
use clap::Subcommand;
use eyre::{Context, ContextCompat};
use std::str::FromStr;
use zksync_ethers_rs::{
    abi::{Hash, Tokenize},
    contracts::governance::{Call, Governance, Operation},
    providers::Middleware,
    types::{Address, Bytes, U256},
};

#[derive(Subcommand)]
pub(crate) enum Command {
    #[clap(
        about = "Returns whether an id corresponds to a registered operation. This includes Waiting, Ready, and Done operations."
    )]
    IsOperation { operation_id: Hash },
    #[clap(
        about = "Returns whether an operation is pending or not. Note that a \"pending\" operation may also be \"ready\"."
    )]
    IsOperationPending { operation_id: Hash },
    #[clap(
        about = "Returns whether an operation is ready for execution. Note that a \"ready\" operation is also \"pending\"."
    )]
    IsOperationReady { operation_id: Hash },
    #[clap(about = "Returns whether an operation is done or not.")]
    IsOperationDone { operation_id: Hash },
    #[clap(about = "Returns the state of an operation.")]
    OperationState { operation_id: Hash },
    #[clap(
        about = "Propose an upgrade, this could be fully transparent providing upgrade data on-chain, or a \"shadow\" upgrade not publishing data on-chain. Only the current owner can propose a shadow upgrade."
    )]
    ProposeUpgrade {
        #[clap(short = 's', long, conflicts_with_all = &["transparent", "operation"], requires = "operation_id", required_unless_present = "transparent")]
        shadow: bool,
        #[clap(long, conflicts_with_all = &["transparent", "operation"], requires = "shadow", required_unless_present = "transparent",)]
        operation_id: Option<Hash>,
        #[clap(long, default_value = "0", value_parser = U256::from_dec_str, required = false)]
        delay: U256,
        #[clap(short = 't', long, conflicts_with_all = &["shadow", "operation_id"], requires = "operation", required_unless_present = "shadow")]
        transparent: bool,
        #[clap(long, conflicts_with_all = &["shadow", "operation_id"], value_parser = parse_operation, requires = "transparent", required = false)]
        operation: Option<Operation>,
        #[clap(long, required = false)]
        explorer_url: bool,
    },
    #[clap(about = "Cancel a scheduled operation.")]
    Cancel { operation_id: Hash },
    #[clap(about = "Execute a scheduled operation.")]
    Execute {
        #[clap(value_parser = parse_operation)]
        operation: Operation,
        #[arg(short = 'i', long, required = false)]
        instant: bool,
        #[arg(short = 'e', long, required = false)]
        explorer_url: bool,
    },
    #[clap(about = "Get the hash of an operation.")]
    HashOperation {
        #[clap(value_parser = parse_operation)]
        operation: Operation,
    },
    #[clap(
        about = "Changes the minimum timelock duration for future operations.",
        visible_alias = "ud"
    )]
    UpdateMinDelay {
        #[clap(required = true)]
        new_min_delay: U256,
        #[clap(default_value = "0", value_parser = U256::from_dec_str, required = false)]
        delay: U256,
        #[arg(short = 's', long, required_unless_present = "transparent_upgrade")]
        shadow_upgrade: bool,
        #[arg(short = 't', long, required_unless_present = "shadow_upgrade")]
        transparent_upgrade: bool,
        #[arg(short = 'e', long, required = false)]
        execute: bool,
        #[arg(long, required = false)]
        explorer_url: bool,
    },
    #[clap(
        about = "Updates the address of the security council.",
        visible_alias = "usc"
    )]
    UpdateSecurityCouncil {
        #[clap(required = true)]
        new_security_council: Address,
        #[clap(default_value = "0", value_parser = U256::from_dec_str, required = false)]
        delay: U256,
        #[arg(short = 's', long, required_unless_present = "transparent_upgrade")]
        shadow_upgrade: bool,
        #[arg(short = 't', long, required_unless_present = "shadow_upgrade")]
        transparent_upgrade: bool,
        #[arg(short = 'e', long, required = false)]
        execute: bool,
        #[arg(long, required = false)]
        explorer_url: bool,
    },
}

#[derive(Debug)]
enum OperationState {
    Unset,
    Waiting,
    Ready,
    Done,
}

impl From<u8> for OperationState {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Unset,
            1 => Self::Waiting,
            2 => Self::Ready,
            3 => Self::Done,
            _ => unreachable!(),
        }
    }
}

impl Command {
    pub async fn run(self, cfg: ZKSyncConfig) -> eyre::Result<()> {
        let governance = try_governance_from_config(&cfg).await?;
        match self {
            Command::IsOperation { operation_id } => {
                let is_operation_pending = governance
                    .is_operation_pending(operation_id.into())
                    .call()
                    .await?;
                println!(
                    "Operation is {:?}",
                    if is_operation_pending {
                        "pending"
                    } else {
                        "not pending"
                    }
                );
            }
            Command::IsOperationPending { operation_id } => {
                let is_operation_pending = governance
                    .is_operation_pending(operation_id.into())
                    .call()
                    .await?;
                println!(
                    "Operation is {:?}",
                    if is_operation_pending {
                        "pending"
                    } else {
                        "not pending"
                    }
                );
            }
            Command::IsOperationReady { operation_id } => {
                let is_operation_ready = governance
                    .is_operation_ready(operation_id.into())
                    .call()
                    .await?;
                println!(
                    "Operation is {:?}",
                    if is_operation_ready {
                        "ready"
                    } else {
                        "not ready"
                    }
                );
            }
            Command::IsOperationDone { operation_id } => {
                let is_operation_done = governance
                    .is_operation_done(operation_id.into())
                    .call()
                    .await?;
                println!(
                    "Operation is {:?}",
                    if is_operation_done { "done" } else { "pending" }
                );
            }
            Command::OperationState { operation_id } => {
                let operation_state: OperationState = governance
                    .get_operation_state(operation_id.into())
                    .call()
                    .await?
                    .into();
                println!("{operation_state:?}");
            }
            Command::ProposeUpgrade {
                shadow,
                operation_id,
                delay,
                transparent,
                operation,
                explorer_url,
            } => {
                let transaction_receipt = if shadow {
                    governance
                        .schedule_shadow(
                            operation_id
                                .context("--operation-id is required with --shadow")?
                                .into(),
                            delay,
                        )
                        .send()
                        .await?
                        .await?
                        .context("No transaction receipt for shadow operation")?
                } else if transparent {
                    governance
                        .schedule_transparent(
                            operation.context("--operation is required with --transparent")?,
                            delay,
                        )
                        .send()
                        .await?
                        .await?
                        .context("No transaction receipt for transparent operation")?
                } else {
                    eyre::bail!("Either --shadow or --transparent must be provided");
                };
                if explorer_url {
                    let url = cfg
                        .network
                        .l1_explorer_url
                        .context("L1 Explorer URL missing in config")?;
                    println!(
                        "Upgrade scheduled: {url}/tx/{:?}",
                        transaction_receipt.transaction_hash
                    );
                } else {
                    println!(
                        "Upgrade scheduled: {:?}",
                        transaction_receipt.transaction_hash
                    );
                }
            }
            Command::Cancel { operation_id } => {
                let transaction_receipt = governance
                    .cancel(operation_id.into())
                    .send()
                    .await?
                    .await?
                    .context("No transaction receipt for operation cancel")?;
                println!(
                    "Operation canceled: {:?}",
                    transaction_receipt.transaction_hash
                );
            }
            Command::Execute {
                operation,
                instant,
                explorer_url,
            } => {
                let transaction_receipt = if instant {
                    governance
                        .execute_instant(operation)
                        .send()
                        .await?
                        .await?
                        .context("No transaction receipt for operation execution")?
                } else {
                    governance
                        .execute(operation)
                        .send()
                        .await?
                        .await?
                        .context("No transaction receipt for operation execution")?
                };
                if explorer_url {
                    let url = cfg
                        .network
                        .l1_explorer_url
                        .context("L1 Explorer URL missing in config")?;
                    println!(
                        "Upgrade executed: {url}/tx/{:?}",
                        transaction_receipt.transaction_hash
                    );
                } else {
                    println!(
                        "Upgrade executed: {:?}",
                        transaction_receipt.transaction_hash
                    );
                }
            }
            Command::HashOperation { operation } => {
                let hashed_operation: Hash =
                    governance.hash_operation(operation).call().await?.into();
                println!("Operation hash: {hashed_operation:?}");
            }
            Command::UpdateMinDelay {
                new_min_delay,
                delay,
                shadow_upgrade,
                transparent_upgrade,
                execute,
                explorer_url,
            } => {
                // Prepare the security council update operation
                let update_delay_calldata = governance
                    .update_delay(new_min_delay)
                    .function
                    .encode_input(&new_min_delay.into_tokens())?;
                run_upgrade(
                    update_delay_calldata.into(),
                    shadow_upgrade || !transparent_upgrade,
                    execute,
                    delay,
                    explorer_url,
                    governance,
                    cfg,
                )
                .await?;
            }
            Command::UpdateSecurityCouncil {
                new_security_council,
                delay,
                shadow_upgrade,
                transparent_upgrade,
                execute,
                explorer_url,
            } => {
                let update_security_council_calldata = governance
                    .update_security_council(new_security_council)
                    .function
                    .encode_input(&new_security_council.into_tokens())?;
                run_upgrade(
                    update_security_council_calldata.into(),
                    shadow_upgrade || !transparent_upgrade,
                    execute,
                    delay,
                    explorer_url,
                    governance,
                    cfg,
                )
                .await?;
            }
        };
        Ok(())
    }
}

pub(crate) fn parse_operation(raw_operation: &str) -> eyre::Result<Operation> {
    let raw_operation = serde_json::Value::from_str(raw_operation).context("Invalid JSON")?;
    let calls = raw_operation
        .get("calls")
        .context("No \"calls\" in operation")?
        .as_array()
        .context("\"calls\" is not an array")?
        .iter()
        .map(|raw_call| {
            let target = raw_call.get("target").context("No target in call")?;
            let value = raw_call
                .get("value")
                .context("No \"value\" in call")?
                .as_u64()
                .context("\"value\" is not a number")?;
            let data = raw_call.get("data").context("No data in call")?;
            Ok(Call {
                target: serde_json::from_value(target.clone())?,
                value: value.into(),
                data: serde_json::from_value(data.clone())?,
            })
        })
        .collect::<Result<Vec<Call>, eyre::Error>>()?;
    // TODO: parse predecessor and salt
    let predecessor = [0_u8; 32];
    let salt = [0_u8; 32];
    let parsed_operation = Operation {
        calls,
        predecessor,
        salt,
    };
    Ok(parsed_operation)
}

pub(crate) async fn run_upgrade(
    calldata: Bytes,
    is_shadow_upgrade: bool,
    execute_upgrade: bool,
    delay: U256,
    explorer_url: bool,
    governance: Governance<impl Middleware + 'static>,
    cfg: ZKSyncConfig,
) -> eyre::Result<()> {
    Box::pin(async {
        let call = Call {
            target: governance.address(),
            value: U256::zero(),
            data: calldata,
        };
        let operation = Operation {
            calls: vec![call],
            predecessor: [0_u8; 32],
            salt: [0_u8; 32],
        };
        let operation_hash = governance.hash_operation(operation.clone()).call().await?;

        // Propose the new security council update
        if is_shadow_upgrade {
            propose_shadow_upgrade(operation_hash, delay, explorer_url, cfg.clone()).await?;
        } else {
            propose_transparent_upgrade(operation.clone(), delay, explorer_url, cfg.clone())
                .await?;
        }

        // Execute the new security council update if wanted
        if execute_upgrade {
            // TODO: make instant execution an option. This would require
            // the current security council private key to be passed in, as
            // the current security council is the only one that can execute
            // the operation instantly.
            Command::Execute {
                operation,
                instant: false,
                explorer_url,
            }
            .run(cfg)
            .await?;
        }
        Ok(())
    })
    .await
}

async fn propose_shadow_upgrade(
    operation_hash: [u8; 32],
    delay: U256,
    explorer_url: bool,
    cfg: ZKSyncConfig,
) -> eyre::Result<()> {
    Command::ProposeUpgrade {
        shadow: true,
        operation_id: Some(operation_hash.into()),
        delay,
        transparent: false,
        operation: None,
        explorer_url,
    }
    .run(cfg)
    .await
}

async fn propose_transparent_upgrade(
    operation: Operation,
    delay: U256,
    explorer_url: bool,
    cfg: ZKSyncConfig,
) -> eyre::Result<()> {
    Command::ProposeUpgrade {
        shadow: false,
        operation_id: None,
        delay,
        transparent: true,
        operation: Some(operation),
        explorer_url,
    }
    .run(cfg)
    .await
}
