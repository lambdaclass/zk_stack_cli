use crate::cli::ZKSyncWeb3Config;
use clap::Args;
use zksync_web3_rs::providers::Provider;
use zksync_web3_rs::types::{Bytes, H256};
use zksync_web3_rs::zks_provider::ZKSProvider;

#[derive(Args)]
pub(crate) struct GetBytecodeByHashArgs {
    #[clap(long, name = "CONTRACT_BYTECODE_HASH")]
    hash: H256,
}

pub(crate) async fn run(args: GetBytecodeByHashArgs, config: ZKSyncWeb3Config) -> eyre::Result<()> {
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
