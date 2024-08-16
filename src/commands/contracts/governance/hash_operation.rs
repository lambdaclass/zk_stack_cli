use crate::commands::contracts::governance::parse_operation;
use clap::Args as ClapArgs;
use zksync_ethers_rs::{
    contracts::governance::{Governance, Operation},
    providers::Middleware,
};

#[derive(ClapArgs, PartialEq)]
pub(crate) struct Args {
    #[clap(short = 'o', long, value_parser = parse_operation)]
    pub operation: Operation,
}

pub(crate) async fn run(
    args: Args,
    governance: Governance<impl Middleware + 'static>,
) -> eyre::Result<()> {
    let hashed_operation = governance.hash_operation(args.operation).call().await?;
    println!("Operation hash: {hashed_operation:?}");
    Ok(())
}
