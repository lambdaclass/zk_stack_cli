use crate::{commands::contracts::governance::run_upgrade, config::ZKSyncConfig};
use clap::Parser;
use zksync_ethers_rs::{
    abi::Tokenize, contracts::governance::Governance, providers::Middleware, types::U256,
};

#[derive(Parser, PartialEq)]
pub(crate) struct Args {
    #[clap(required = true)]
    pub new_min_delay: U256,
    #[clap(default_value = "0", value_parser = U256::from_dec_str, required = false)]
    pub delay: U256,
    #[arg(short = 's', long, required_unless_present = "transparent_upgrade")]
    pub shadow_upgrade: bool,
    #[arg(short = 't', long, required_unless_present = "shadow_upgrade")]
    pub transparent_upgrade: bool,
    #[arg(short = 'e', long, required = false)]
    pub execute: bool,
    #[arg(long, required = false)]
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
    run_upgrade(
        update_delay_calldata.into(),
        args.shadow_upgrade || !args.transparent_upgrade,
        args.execute,
        args.delay,
        args.explorer_url,
        governance,
        cfg,
    )
    .await?;
    Ok(())
}
