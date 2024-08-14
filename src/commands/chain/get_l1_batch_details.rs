use crate::config::ZKSyncConfig;
use clap::Args as ClapArgs;
use zksync_ethers_rs::providers::Provider;
use zksync_ethers_rs::ZKMiddleware;

#[derive(ClapArgs)]
pub(crate) struct Args {
    #[clap(long, name = "L1_BATCH_NUMBER")]
    batch: u32,
}

pub(crate) async fn run(args: Args, cfg: ZKSyncConfig) -> eyre::Result<()> {
    let provider = Provider::try_from(cfg.network.l2_rpc_url)?;
    let l1_batch_details = provider.get_l1_batch_details(args.batch).await?;
    println!("{l1_batch_details:#?}");
    Ok(())
}
