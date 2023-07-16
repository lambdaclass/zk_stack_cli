use crate::cli::ZKSyncConfig;
use clap::Args as ClapArgs;
use zksync_web3_rs::{
    providers::{Middleware, Provider},
    types::Address,
};

#[derive(ClapArgs)]
pub(crate) struct Args {
    #[clap(short, long, name = "ACCOUNT_ADDRESS")]
    pub account: Address,
}

pub(crate) async fn run(args: Args, config: ZKSyncConfig) -> eyre::Result<()> {
    let provider = Provider::try_from(format!(
        "http://{host}:{port}",
        host = config.host,
        port = config.port
    ))?
    .interval(std::time::Duration::from_millis(10));
    let balance = provider.get_balance(args.account, None).await?;
    log::info!("{:#?}", balance);
    Ok(())
}
