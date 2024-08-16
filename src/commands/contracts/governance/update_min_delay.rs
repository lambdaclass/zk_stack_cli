use crate::{
    commands::contracts::governance::{execute, propose},
    config::ZKSyncConfig,
};
use clap::Args as ClapArgs;
use zksync_ethers_rs::{
    abi::Tokenize,
    contracts::governance::{Call, Governance, Operation},
    providers::Middleware,
    types::{Address, U256},
};

#[derive(ClapArgs, PartialEq)]
pub(crate) struct Args {
    #[clap(long = "delay", required = true)]
    pub new_min_delay: U256,
    #[clap(short = 's', long, required_unless_present = "transparent_upgrade")]
    pub shadow_upgrade: bool,
    #[clap(short = 't', long, required_unless_present = "shadow_upgrade")]
    pub transparent_upgrade: bool,
    #[clap(short = 'e', long, required = false)]
    pub execute: bool,
    #[clap(long, default_value = "0", value_parser = U256::from_dec_str, required = false)]
    pub delay: U256,
    #[clap(long, required = false)]
    pub explorer_url: bool,
}

pub(crate) async fn run(
    args: Args,
    governance: Governance<impl Middleware + 'static>,
    cfg: ZKSyncConfig,
) -> eyre::Result<()> {
    // Prepare the security council update operation
    let update_delay_calldata = governance
        .update_delay(args.new_min_delay)
        .function
        .encode_input(&args.new_min_delay.into_tokens())?;
    let update_delay_call = Call {
        target: governance.address(),
        value: 0.into(),
        data: update_delay_calldata.into(),
    };
    let update_delay_operation = Operation {
        calls: vec![update_delay_call],
        predecessor: [0_u8; 32],
        salt: [0_u8; 32],
    };
    let update_delay_operation_hash = governance
        .hash_operation(update_delay_operation.clone())
        .call()
        .await?;

    // Propose the new security council update
    if args.shadow_upgrade {
        propose_shadow_min_delay_update(
            governance.clone(),
            update_delay_operation_hash,
            args.delay,
            args.explorer_url,
            cfg.clone(),
        )
        .await?;
    }
    if args.transparent_upgrade {
        propose_transparent_min_delay_update(
            governance.clone(),
            update_delay_operation.clone(),
            args.delay,
            args.explorer_url,
            cfg.clone(),
        )
        .await?;
    }

    // Execute the new security council update if wanted
    if args.execute {
        execute::run(
            execute::Args {
                operation: update_delay_operation,
                // TODO: make instant execution an option. This would require
                // the current security council private key to be passed in, as
                // the current security council is the only one that can execute
                // the operation instantly.
                instant: false,
                explorer_url: args.explorer_url,
            },
            governance,
            cfg,
        )
        .await?
    }
    Ok(())
}

async fn propose_shadow_min_delay_update(
    governance: Governance<impl Middleware + 'static>,
    update_delay_operation_hash: [u8; 32],
    delay: U256,
    explorer_url: bool,
    cfg: ZKSyncConfig,
) -> eyre::Result<()> {
    propose::run(
        shadow_propose_upgrade_args(update_delay_operation_hash, delay, explorer_url),
        governance,
        cfg,
    )
    .await
}

fn shadow_propose_upgrade_args(
    update_delay_operation_hash: [u8; 32],
    delay: U256,
    explorer_url: bool,
) -> propose::Args {
    propose::Args {
        shadow: true,
        operation_id: Some(update_delay_operation_hash.into()),
        delay,
        transparent: false,
        operation: None,
        explorer_url,
    }
}

async fn propose_transparent_min_delay_update(
    governance: Governance<impl Middleware + 'static>,
    update_delay_operation: Operation,
    delay: U256,
    explorer_url: bool,
    cfg: ZKSyncConfig,
) -> eyre::Result<()> {
    propose::run(
        transparent_propose_upgrade_args(update_delay_operation, delay, explorer_url),
        governance,
        cfg,
    )
    .await
}

fn transparent_propose_upgrade_args(
    update_delay_operation: Operation,
    delay: U256,
    explorer_url: bool,
) -> propose::Args {
    propose::Args {
        shadow: false,
        operation_id: None,
        delay,
        transparent: true,
        operation: Some(update_delay_operation),
        explorer_url,
    }
}
