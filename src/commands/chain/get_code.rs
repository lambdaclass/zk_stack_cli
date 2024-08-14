use crate::config::ZKSyncConfig;
use clap::Args as ClapArgs;
use zksync_ethers_rs::{
    providers::{Middleware, Provider},
    types::Address,
};

#[derive(ClapArgs)]
pub(crate) struct Args {
    #[clap(long = "address")]
    pub contract: Address,
}

pub(crate) async fn run(args: Args, cfg: ZKSyncConfig) -> eyre::Result<()> {
    let provider = Provider::try_from(cfg.network.l2_rpc_url)?;
    let deployed_bytecode = provider.get_code(args.contract, None).await?;
    println!("{deployed_bytecode:#?}");
    Ok(())
}
