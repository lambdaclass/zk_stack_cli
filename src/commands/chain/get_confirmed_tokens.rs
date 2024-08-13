use crate::config::ZKSyncConfig;
use clap::Args as ClapArgs;
use zksync_ethers_rs::providers::Provider;
use zksync_ethers_rs::ZKMiddleware;

#[derive(ClapArgs)]
pub(crate) struct Args {
    #[clap(long, name = "FROM")]
    from: u32,
    #[clap(long, name = "LIMIT")]
    limit: u8,
}

pub(crate) async fn run(args: Args, cfg: ZKSyncConfig) -> eyre::Result<()> {
    let provider = Provider::try_from(cfg.network.l2_rpc_url)?;
    let confirmed_tokens = provider.get_confirmed_tokens(args.from, args.limit).await?;
    println!("Confirmed Tokens: {confirmed_tokens:#?}");
    Ok(())
}
