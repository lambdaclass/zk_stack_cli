use crate::config::ZKSyncConfig;
use clap::Args as ClapArgs;
use zksync_ethers_rs::providers::Provider;
use zksync_ethers_rs::ZKMiddleware;

#[derive(ClapArgs)]
pub(crate) struct Args {
    #[clap(long = "number")]
    block_number: u32,
}

pub(crate) async fn run(args: Args, cfg: ZKSyncConfig) -> eyre::Result<()> {
    let provider = Provider::try_from(cfg.network.l2_rpc_url)?;
    let block_details = provider.get_block_details(args.block_number).await?;
    if let Some(block_details) = block_details {
        println!("{block_details:#?}");
    } else {
        println!("Block {} not found", args.block_number);
    }
    Ok(())
}
