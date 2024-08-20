use clap::Subcommand;
use eyre::{Context, ContextCompat};
use std::str::FromStr;
use zksync_ethers_rs::{
    contracts::governance::{Call, Governance, Operation},
    providers::Middleware,
    types::{Bytes, U256},
};

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
    #[clap(
        about = "Changes the minimum timelock duration for future operations.",
        visible_alias = "ud"
    )]
    UpdateMinDelay(update_min_delay::Args),
    #[clap(
        about = "Updates the address of the security council.",
        visible_alias = "usc"
    )]
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

pub(crate) async fn run_upgrade(
    calldata: Bytes,
    is_shadow_upgrade: bool,
    execute_upgrade: bool,
    delay: U256,
    explorer_url: bool,
    governance: Governance<impl Middleware + 'static>,
    cfg: ZKSyncConfig,
) -> eyre::Result<()> {
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
        propose_shadow_upgrade(
            governance.clone(),
            operation_hash,
            delay,
            explorer_url,
            cfg.clone(),
        )
        .await?;
    } else {
        propose_transparent_upgrade(
            governance.clone(),
            operation.clone(),
            delay,
            explorer_url,
            cfg.clone(),
        )
        .await?;
    }

    // Execute the new security council update if wanted
    if execute_upgrade {
        execute::run(
            execute::Args {
                operation,
                // TODO: make instant execution an option. This would require
                // the current security council private key to be passed in, as
                // the current security council is the only one that can execute
                // the operation instantly.
                instant: false,
                explorer_url,
            },
            governance,
            cfg,
        )
        .await?
    }
    Ok(())
}

async fn propose_shadow_upgrade(
    governance: Governance<impl Middleware + 'static>,
    operation_hash: [u8; 32],
    delay: U256,
    explorer_url: bool,
    cfg: ZKSyncConfig,
) -> eyre::Result<()> {
    propose::run(
        propose::Args {
            shadow: true,
            operation_id: Some(operation_hash.into()),
            delay,
            transparent: false,
            operation: None,
            explorer_url,
        },
        governance,
        cfg,
    )
    .await
}

async fn propose_transparent_upgrade(
    governance: Governance<impl Middleware + 'static>,
    operation: Operation,
    delay: U256,
    explorer_url: bool,
    cfg: ZKSyncConfig,
) -> eyre::Result<()> {
    propose::run(
        propose::Args {
            shadow: false,
            operation_id: None,
            delay,
            transparent: true,
            operation: Some(operation),
            explorer_url,
        },
        governance,
        cfg,
    )
    .await
}
