use clap::Parser;
use zksync_ethers_rs::{abi::Hash, contracts::governance::Governance, providers::Middleware};

#[derive(Parser, PartialEq)]
pub(crate) struct Args {
    pub operation_id: Hash,
}

pub(crate) async fn run(
    args: Args,
    governance: Governance<impl Middleware + 'static>,
) -> eyre::Result<()> {
    let is_operation = governance
        .is_operation(args.operation_id.into())
        .call()
        .await?;
    println!(
        "This operation is {:?}",
        if is_operation {
            "registered"
        } else {
            "not registered"
        }
    );
    Ok(())
}
