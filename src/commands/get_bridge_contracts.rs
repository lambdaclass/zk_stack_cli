use crate::cli::ZKSyncWeb3Config;
use crate::zks_provider::ZKSProvider;
use crate::providers::Provider;
use clap::Args;

#[derive(Args)]
pub(crate) struct GetBridgeContracts;

pub(crate) async fn run(config: ZKSyncWeb3Config) -> eyre::Result<()> {
    let provider = Provider::try_from(format!(
        "http://{host}:{port}",
        host = config.host,
        port = config.port
    ))?
    .interval(std::time::Duration::from_millis(10));
    let bridge_contracts = provider
        .get_bridge_contracts()
        .await?;
    log::info!("L1 ERC20 default bridge contract: {:#?}", bridge_contracts.l1_erc20_default_bridge);
    log::info!("L2 ERC20 default bridge contract: {:#?}", bridge_contracts.l2_erc20_default_bridge);
    Ok(())
}
