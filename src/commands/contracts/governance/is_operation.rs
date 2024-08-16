use clap::Args as ClapArgs;
use zksync_ethers_rs::{abi::Hash, contracts::governance::Governance, providers::Middleware};

#[derive(ClapArgs, PartialEq)]
pub(crate) struct Args {
    #[clap(short = 'o', long, index = 0)]
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
