use crate::{commands::contracts::governance::parse_operation, config::ZKSyncConfig};
use clap::Args as ClapArgs;
use eyre::ContextCompat;
use zksync_ethers_rs::{
    contracts::governance::{Governance, Operation},
    providers::Middleware,
};

#[derive(ClapArgs, PartialEq)]
pub(crate) struct Args {
    #[clap(long, conflicts_with_all = &["shadow", "operation_id"], value_parser = parse_operation, index = 0)]
    pub operation: Operation,
    #[clap(short = 'i', long, required = false)]
    pub instant: bool,
    #[clap(long, required = false)]
    pub explorer_url: bool,
}

pub(crate) async fn run(
    args: Args,
    governance: Governance<impl Middleware + 'static>,
    cfg: ZKSyncConfig,
) -> eyre::Result<()> {
    let transaction_receipt = if args.instant {
        governance
            .execute_instant(args.operation)
            .send()
            .await?
            .await?
            .context("No transaction receipt for operation execution")?
    } else {
        governance
            .execute(args.operation)
            .send()
            .await?
            .await?
            .context("No transaction receipt for operation execution")?
    };
    if args.explorer_url {
        let url = cfg
            .network
            .l1_explorer_url
            .context("L1 Explorer URL missing in config")?;
        println!(
            "Upgrade executed: {url}/tx/{:?}",
            transaction_receipt.transaction_hash
        );
    } else {
        println!(
            "Upgrade executed: {:?}",
            transaction_receipt.transaction_hash
        );
    }
    Ok(())
}
