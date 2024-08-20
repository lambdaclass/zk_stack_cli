use crate::commands::contracts::governance::parse_operation;
use clap::Parser;
use zksync_ethers_rs::{
    abi::Hash,
    contracts::governance::{Governance, Operation},
    providers::Middleware,
};

#[derive(Parser, PartialEq)]
pub(crate) struct Args {
    #[clap(value_parser = parse_operation)]
    pub operation: Operation,
}

pub(crate) async fn run(
    args: Args,
    governance: Governance<impl Middleware + 'static>,
) -> eyre::Result<()> {
    let hashed_operation: Hash = governance
        .hash_operation(args.operation)
        .call()
        .await?
        .into();
    println!("Operation hash: {hashed_operation:?}");
    Ok(())
}
