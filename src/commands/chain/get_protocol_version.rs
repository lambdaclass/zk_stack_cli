use crate::config::ZKSyncConfig;
use clap::Args as ClapArgs;
use zksync_ethers_rs::providers::Provider;
use zksync_ethers_rs::ZKMiddleware;

#[derive(ClapArgs, PartialEq)]
pub(crate) struct Args {
    #[clap(long)]
    id: Option<u16>,
}

pub(crate) async fn run(args: Args, cfg: ZKSyncConfig) -> eyre::Result<()> {
    let provider = Provider::try_from(cfg.network.l2_rpc_url)?;
    let protocol_version = provider.get_protocol_version(args.id).await?;
    if let Some(protocol_version) = protocol_version {
        println!("{protocol_version:#?}");
    } else {
        println!("Protocol version not found");
    }
    Ok(())
}
