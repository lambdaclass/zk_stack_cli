use crate::config::ZKSyncConfig;
use zksync_ethers_rs::providers::Provider;
use zksync_ethers_rs::ZKMiddleware;

pub(crate) async fn run(config: ZKSyncConfig) -> eyre::Result<()> {
    let provider = Provider::try_from(config.l2_rpc_url)?;
    let l1_chain_id = provider.get_l1_chain_id().await?;
    println!("L1 Chain ID: {l1_chain_id:#?}");
    Ok(())
}
