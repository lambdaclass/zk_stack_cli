use crate::cli::ZKSyncConfig;
use zksync_web3_rs::providers::Provider;
use zksync_web3_rs::zks_provider::ZKSProvider;

pub(crate) async fn run(config: ZKSyncConfig) -> eyre::Result<()> {
    let provider = if let Some(port) = config.l2_port {
        Provider::try_from(format!("http://{host}:{port}", host = config.host))?
    } else {
        Provider::try_from(config.host.clone())?
    }
    .interval(std::time::Duration::from_millis(10));
    let bridge_contracts = provider.get_bridge_contracts().await?;
    log::info!(
        "L1 ERC20 default bridge contract: {:#?}",
        bridge_contracts.l1_erc20_default_bridge
    );
    log::info!(
        "L2 ERC20 default bridge contract: {:#?}",
        bridge_contracts.l2_erc20_default_bridge
    );
    Ok(())
}
