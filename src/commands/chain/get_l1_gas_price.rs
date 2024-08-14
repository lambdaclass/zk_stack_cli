use crate::config::ZKSyncConfig;
use zksync_ethers_rs::providers::Provider;
use zksync_ethers_rs::ZKMiddleware;

pub(crate) async fn run(cfg: ZKSyncConfig) -> eyre::Result<()> {
    let provider = Provider::try_from(cfg.network.l2_rpc_url)?;
    let current_l1_gas_price = provider.get_l1_gas_price().await?;
    println!("Current L1 Gas Price (wei): {current_l1_gas_price:#?}");
    Ok(())
}
