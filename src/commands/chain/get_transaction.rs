use crate::config::ZKSyncConfig;
use clap::Args as ClapArgs;
use eyre::ContextCompat;
use zksync_ethers_rs::{
    providers::{Middleware, Provider},
    types::H256,
};

#[derive(ClapArgs)]
pub(crate) struct Args {
    #[clap(short, long, name = "TRANSACTION_HASH")]
    pub transaction: H256,
}

pub(crate) async fn run(args: Args, cfg: ZKSyncConfig) -> eyre::Result<()> {
    let provider = Provider::try_from(cfg.network.l2_rpc_url)?;
    let transaction = provider
        .get_transaction(args.transaction)
        .await?
        .context("No pending transaction")?;
    log::info!("{:#?}", transaction);
    Ok(())
}
