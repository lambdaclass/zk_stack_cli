use crate::config::ZKSyncConfig;
use eyre::ContextCompat;

pub(crate) async fn run(cfg: ZKSyncConfig) -> eyre::Result<()> {
    let wallet_config = cfg.wallet.context("Wallet config missing")?;
    println!("Wallet address: {:?}", wallet_config.address);
    Ok(())
}
