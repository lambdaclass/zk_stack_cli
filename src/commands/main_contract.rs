use crate::cli::ZKSyncConfig;
use zksync_web3_rs::providers::Provider;
use zksync_web3_rs::zks_provider::ZKSProvider;

pub(crate) async fn run(config: ZKSyncConfig) -> eyre::Result<()> {
    let provider = if let Some(port) = config.l2_port {
        Provider::try_from(format!(
            "http://{host}:{port}",
            host = config.host,
            port = port
        ))?
    } else {
        Provider::try_from(config.host.to_owned())?
    }
    .interval(std::time::Duration::from_millis(10));
    let main_contract = provider.get_main_contract().await?;
    log::info!("{main_contract:#?}");
    Ok(())
}
