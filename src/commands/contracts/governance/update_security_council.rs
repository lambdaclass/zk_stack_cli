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
    #[clap(long = "address", required = true)]
    pub new_security_council: Address,
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
    let update_security_council_calldata = governance
        .update_security_council(args.new_security_council)
        .function
        .encode_input(&args.new_security_council.into_tokens())?;
    let update_security_council_call = Call {
        target: governance.address(),
        value: U256::zero(),
        data: update_security_council_calldata.into(),
    };
    let update_security_council_operation = Operation {
        calls: vec![update_security_council_call],
        predecessor: [0_u8; 32],
        salt: [0_u8; 32],
    };
    let update_security_council_operation_hash = governance
        .hash_operation(update_security_council_operation.clone())
        .call()
        .await?;

    // Propose the new security council update
    if args.shadow_upgrade {
        propose_shadow_security_council_update(
            governance.clone(),
            update_security_council_operation_hash,
            args.delay,
            args.explorer_url,
            cfg.clone(),
        )
        .await?;
    }
    if args.transparent_upgrade {
        propose_transparent_security_council_update(
            governance.clone(),
            update_security_council_operation.clone(),
            args.delay,
            args.explorer_url,
            cfg.clone(),
        )
        .await?;
    }

    // Execute the new security council update
    if args.execute {
        execute::run(
            execute::Args {
                operation: update_security_council_operation,
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

fn shadow_propose_upgrade_args(
    update_security_council_operation_hash: [u8; 32],
    delay: U256,
    explorer_url: bool,
) -> propose::Args {
    propose::Args {
        shadow: true,
        operation_id: Some(update_security_council_operation_hash.into()),
        delay,
        transparent: false,
        operation: None,
        explorer_url,
    }
}

fn transparent_propose_upgrade_args(
    update_security_council_operation: Operation,
    delay: U256,
    explorer_url: bool,
) -> propose::Args {
    propose::Args {
        shadow: false,
        operation_id: None,
        delay,
        transparent: true,
        operation: Some(update_security_council_operation),
        explorer_url,
    }
}

async fn propose_shadow_security_council_update(
    governance: Governance<impl Middleware + 'static>,
    update_security_council_operation_hash: [u8; 32],
    delay: U256,
    explorer_url: bool,
    cfg: ZKSyncConfig,
) -> eyre::Result<()> {
    propose::run(
        shadow_propose_upgrade_args(update_security_council_operation_hash, delay, explorer_url),
        governance,
        cfg,
    )
    .await
}

async fn propose_transparent_security_council_update(
    governance: Governance<impl Middleware + 'static>,
    update_security_council_operation: Operation,
    delay: U256,
    explorer_url: bool,
    cfg: ZKSyncConfig,
) -> eyre::Result<()> {
    propose::run(
        transparent_propose_upgrade_args(update_security_council_operation, delay, explorer_url),
        governance,
        cfg,
    )
    .await
}
