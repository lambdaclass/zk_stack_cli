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
    let main_contract_address = provider.get_main_contract().await?;
    print!("Main Contract: ");
    if args.explorer_url && cfg.network.l2_explorer_url.is_some() {
        println!(
            "{}/address/{main_contract_address:#?}",
            cfg.network.l2_explorer_url.unwrap()
        );
    } else {
        println!("{main_contract_address:#?}");
    }
    Ok(())
}
