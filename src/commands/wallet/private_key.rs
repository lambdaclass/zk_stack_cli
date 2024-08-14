use crate::config::ZKSyncConfig;
use eyre::ContextCompat;

pub(crate) async fn run(cfg: ZKSyncConfig) -> eyre::Result<()> {
    let wallet_config = cfg.wallet.context("Wallet config missing")?;
    println!("Wallet private key: {:?}", wallet_config.private_key);
    Ok(())
}
