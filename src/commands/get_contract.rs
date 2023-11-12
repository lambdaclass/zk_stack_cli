use crate::cli::ZKSyncConfig;
use clap::Args as ClapArgs;
use zksync_web3_rs::{
    providers::{Middleware, Provider},
    types::Address,
};

#[derive(ClapArgs)]
pub(crate) struct Args {
    #[clap(short, long, name = "CONTRACT_ADDRESS")]
    pub contract: String,
}

pub(crate) async fn run(args: Args, config: ZKSyncConfig) -> eyre::Result<()> {
    let provider = if let Some(port) = config.l2_port {
        Provider::try_from(format!("http://{host}:{port}", host = config.host))?
    } else {
        Provider::try_from(config.host.clone())?
    }
    .interval(std::time::Duration::from_millis(10));
    let contract = provider
        .get_code(args.contract.parse::<Address>()?, None)
        .await?;
    log::info!("{:#?}", contract);
    Ok(())
}
