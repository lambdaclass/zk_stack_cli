use crate::cli::ZKSyncConfig;
use clap::Args as ClapArgs;
use zksync_web3_rs::providers::Provider;
use zksync_web3_rs::zks_provider::ZKSProvider;

#[derive(ClapArgs)]
pub(crate) struct Args {
    #[clap(long, name = "FROM")]
    from: u32,
    #[clap(long, name = "LIMIT")]
    limit: u8,
}

pub(crate) async fn run(args: Args, config: ZKSyncConfig) -> eyre::Result<()> {
    let provider = Provider::try_from(format!(
        "http://{host}:{port}",
        host = config.host,
        port = config.l2_port
    ))?
    .interval(std::time::Duration::from_millis(10));
    let confirmed_tokens = provider.get_confirmed_tokens(args.from, args.limit).await?;
    log::info!("{:#?}", confirmed_tokens);
    Ok(())
}
