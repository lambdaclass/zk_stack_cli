use crate::config::ZKSyncConfig;
use clap::Args as ClapArgs;
use eyre::ContextCompat;
use zksync_ethers_rs::providers::Provider;
use zksync_ethers_rs::ZKMiddleware;

#[derive(ClapArgs, PartialEq)]
pub(crate) struct Args {
    #[clap(long, default_value_t = false)]
    explorer_url: bool,
}

pub(crate) async fn run(args: Args, cfg: ZKSyncConfig) -> eyre::Result<()> {
    let provider = Provider::try_from(cfg.network.l2_rpc_url)?;
    let testnet_paymaster_address = provider.get_testnet_paymaster().await?;
    print!("Testnet Paymaster Address: ");
    if args.explorer_url && cfg.network.l2_explorer_url.is_some() {
        println!(
            "{}/address/{testnet_paymaster_address:#?}",
            cfg.network
                .l2_explorer_url
                .context("L2 Explorer URL missing in config")?,
        );
    } else {
        println!("{testnet_paymaster_address:#?}");
    }
    Ok(())
}
