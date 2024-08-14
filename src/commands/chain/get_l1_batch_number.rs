use crate::config::ZKSyncConfig;
use zksync_ethers_rs::{providers::Provider, ZKMiddleware};

pub(crate) async fn run(cfg: ZKSyncConfig) -> eyre::Result<()> {
    let provider = Provider::try_from(cfg.network.l2_rpc_url)?;
    let l1_batch_number = provider.get_l1_batch_number().await?;
    println!("Latest L1 Batch Number: {l1_batch_number:#?}");
    Ok(())
}
