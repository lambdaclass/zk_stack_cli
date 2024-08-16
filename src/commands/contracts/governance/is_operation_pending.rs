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
    let is_operation_pending = governance
        .is_operation_pending(args.operation_id.into())
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
    Ok(())
}
