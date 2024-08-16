use std::str::FromStr;

use clap::Subcommand;
use eyre::{Context, ContextCompat};
use zksync_ethers_rs::contracts::governance::{Call, Operation};

use crate::{commands::utils::try_governance_from_config, config::ZKSyncConfig};

pub(crate) mod cancel;
pub(crate) mod execute;
pub(crate) mod hash_operation;
pub(crate) mod is_operation;
pub(crate) mod is_operation_done;
pub(crate) mod is_operation_pending;
pub(crate) mod is_operation_ready;
pub(crate) mod operation_state;
pub(crate) mod propose;
pub(crate) mod update_min_delay;
pub(crate) mod update_security_council;

#[derive(Subcommand, PartialEq)]
pub(crate) enum Command {
    #[clap(
        about = "Returns whether an id corresponds to a registered operation. This includes Waiting, Ready, and Done operations."
    )]
    IsOperation(is_operation::Args),
    #[clap(
        about = "Returns whether an operation is pending or not. Note that a \"pending\" operation may also be \"ready\"."
    )]
    IsOperationPending(is_operation_pending::Args),
    #[clap(
        about = "Returns whether an operation is ready for execution. Note that a \"ready\" operation is also \"pending\"."
    )]
    IsOperationReady(is_operation_ready::Args),
    #[clap(about = "Returns whether an operation is done or not.")]
    IsOperationDone(is_operation_done::Args),
    #[clap(about = "Returns the state of an operation.")]
    OperationState(operation_state::Args),
    #[clap(
        about = "Propose an upgrade, this could be fully transparent providing upgrade data on-chain, or a \"shadow\" upgrade not publishing data on-chain. Only the current owner can propose a shadow upgrade."
    )]
    ProposeUpgrade(propose::Args),
    #[clap(about = "Cancel a scheduled operation.")]
    Cancel(cancel::Args),
    #[clap(about = "Execute a scheduled operation.")]
    Execute(execute::Args),
    #[clap(about = "Get the hash of an operation.")]
    HashOperation(hash_operation::Args),
    #[clap(about = "Changes the minimum timelock duration for future operations.")]
    UpdateMinDelay(update_min_delay::Args),
    #[clap(about = "Updates the address of the security council.")]
    UpdateSecurityCouncil(update_security_council::Args),
}

pub(crate) async fn start(cmd: Command, cfg: ZKSyncConfig) -> eyre::Result<()> {
    let governance = try_governance_from_config(&cfg).await?;
    match cmd {
        Command::IsOperation(args) => is_operation::run(args, governance).await?,
        Command::IsOperationPending(args) => is_operation_pending::run(args, governance).await?,
        Command::IsOperationReady(args) => is_operation_ready::run(args, governance).await?,
        Command::IsOperationDone(args) => is_operation_done::run(args, governance).await?,
        Command::OperationState(args) => operation_state::run(args, governance).await?,
        Command::ProposeUpgrade(args) => propose::run(args, governance, cfg).await?,
        Command::Cancel(args) => cancel::run(args, governance).await?,
        Command::Execute(args) => execute::run(args, governance, cfg).await?,
        Command::HashOperation(args) => hash_operation::run(args, governance).await?,
        Command::UpdateMinDelay(args) => update_min_delay::run(args, governance, cfg).await?,
        Command::UpdateSecurityCouncil(args) => {
            update_security_council::run(args, governance, cfg).await?
        }
    };
    Ok(())
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
