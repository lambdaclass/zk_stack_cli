use crate::cli::ZKSyncConfig;
use clap::Args as ClapArgs;
use zksync_web3_rs::providers::Provider;
use zksync_web3_rs::types::U64;
use zksync_web3_rs::zks_provider::ZKSProvider;

#[derive(ClapArgs)]
pub(crate) struct Args {
    #[clap(long, name = "L1_BATCH_NUMBER")]
    batch: U64,
}

pub(crate) async fn run(args: Args, config: ZKSyncConfig) -> eyre::Result<()> {
    let provider = Provider::try_from(format!(
        "http://{host}:{port}",
        host = config.host,
        port = config.port
    ))?
    .interval(std::time::Duration::from_millis(10));
    let l1_batch_details = provider.get_l1_batch_details(args.batch).await?;
    log::info!("{:#?}", l1_batch_details);
    Ok(())
}
