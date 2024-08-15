use crate::config::ZKSyncConfig;
use clap::Args as ClapArgs;
use eyre::ContextCompat;
use zksync_ethers_rs::{providers::Provider, types::H256, ZKMiddleware};

#[derive(ClapArgs, PartialEq)]
pub(crate) struct Args {
    #[clap(long = "hash", name = "TRANSACTION_HASH")]
    pub transaction_hash: H256,
}

pub(crate) async fn run(args: Args, cfg: ZKSyncConfig) -> eyre::Result<()> {
    let provider = Provider::try_from(cfg.network.l2_rpc_url)?;
    let transaction_details = provider
        .get_transaction_details(args.transaction_hash)
        .await?
        .context("No pending transaction")?;
    log::info!("{transaction_details:#?}");
    Ok(())
}
