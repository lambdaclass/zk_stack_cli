use crate::cli::ZKSyncConfig;
use clap::Args as ClapArgs;
use zksync_web3_rs::providers::Provider;
use zksync_web3_rs::types::{Bytes, H256};
use zksync_web3_rs::zks_provider::ZKSProvider;

#[derive(ClapArgs)]
pub(crate) struct Args {
    #[clap(long, name = "CONTRACT_BYTECODE_HASH")]
    hash: H256,
}

pub(crate) async fn run(args: Args, config: ZKSyncConfig) -> eyre::Result<()> {
    let provider = Provider::try_from(format!(
        "http://{host}:{port}",
        host = config.host,
        port = config.port
    ))?
    .interval(std::time::Duration::from_millis(10));
    let contract_bytecode = provider
        .get_bytecode_by_hash(args.hash)
        .await?
        .map(Bytes::from);
    if let Some(contract_bytecode) = contract_bytecode {
        log::info!("{:#?}", contract_bytecode);
    } else {
        log::info!("0x");
    }
    Ok(())
}