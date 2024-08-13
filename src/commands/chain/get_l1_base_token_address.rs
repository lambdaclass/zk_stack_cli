use crate::config::ZKSyncConfig;
use clap::Args as ClapArgs;
use zksync_ethers_rs::providers::Provider;
use zksync_ethers_rs::ZKMiddleware;

#[derive(ClapArgs)]
pub(crate) struct Args {
    #[clap(long, default_value_t = false)]
    explorer_url: bool,
}

pub(crate) async fn run(args: Args, cfg: ZKSyncConfig) -> eyre::Result<()> {
    let provider = Provider::try_from(cfg.network.l2_rpc_url)?;
    let l1_base_token_address = provider.get_base_token_l1_address().await?;
    print!("L1 Base Token Address: ");
    if args.explorer_url && cfg.network.l1_explorer_url.is_some() {
        println!(
            "{}/address/{l1_base_token_address:#?}",
            cfg.network.l1_explorer_url.unwrap()
        );
    } else {
        println!("{l1_base_token_address:#?}");
    }
    Ok(())
}
