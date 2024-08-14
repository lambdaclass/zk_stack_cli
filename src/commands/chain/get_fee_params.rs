use crate::config::ZKSyncConfig;
use zksync_ethers_rs::providers::Provider;
use zksync_ethers_rs::ZKMiddleware;

pub(crate) async fn run(cfg: ZKSyncConfig) -> eyre::Result<()> {
    let provider = Provider::try_from(cfg.network.l2_rpc_url)?;
    let fee_params = provider.get_fee_params().await?;
    println!("{fee_params:#?}");
    Ok(())
}
