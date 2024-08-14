use crate::config::ZKSyncConfig;
use clap::Args as ClapArgs;
use eyre::ContextCompat;
use zksync_ethers_rs::abi::Hash;
use zksync_ethers_rs::deposit;
use zksync_ethers_rs::providers::Provider;

#[derive(ClapArgs)]
pub(crate) struct Args {
    #[clap(long = "hash")]
    l1_deposit_tx_hash: Hash,
    #[clap(long, required = false)]
    explorer_url: bool,
}

pub(crate) async fn run(args: Args, cfg: ZKSyncConfig) -> eyre::Result<()> {
    let provider = Provider::try_from(
        cfg.network
            .l1_rpc_url
            .context("L1 RPC URL missing in config")?,
    )?;
    let deposit_finalization_hash =
        deposit::l2_deposit_tx_hash(args.l1_deposit_tx_hash, &provider).await;
    if args.explorer_url {
        let url = cfg
            .network
            .l2_explorer_url
            .context("L2 Explorer URL missing in config")?;
        println!(
            "Deposit finalization: {}/tx/{deposit_finalization_hash:#?}",
            url
        );
    } else {
        println!("Deposit finalization hash: {deposit_finalization_hash:#?}");
    }
    Ok(())
}
