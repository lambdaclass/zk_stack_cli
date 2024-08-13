use crate::config::ZKSyncConfig;
use clap::Args as ClapArgs;
use zksync_ethers_rs::{providers::Provider, types::Address, ZKMiddleware};

#[derive(ClapArgs)]
pub(crate) struct Args {
    #[clap(long = "address")]
    pub account_address: Address,
}

pub(crate) async fn run(args: Args, cfg: ZKSyncConfig) -> eyre::Result<()> {
    let provider = Provider::try_from(cfg.network.l2_rpc_url)?;
    let all_account_balances = provider
        .get_all_account_balances(args.account_address)
        .await?;
    println!("{all_account_balances:#?}");
    Ok(())
}
