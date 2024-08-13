use crate::config::ZKSyncConfig;
use clap::Args as ClapArgs;
use zksync_ethers_rs::providers::Provider;
use zksync_ethers_rs::types::{Bytes, H256};
use zksync_ethers_rs::ZKMiddleware;

#[derive(ClapArgs)]
pub(crate) struct Args {
    #[clap(long, name = "CONTRACT_BYTECODE_HASH")]
    hash: H256,
}

pub(crate) async fn run(args: Args, config: ZKSyncConfig) -> eyre::Result<()> {
    let provider = Provider::try_from(config.l2_rpc_url)?;
    let contract_bytecode = provider
        .get_bytecode_by_hash(args.hash)
        .await?
        .map(Bytes::from);
    if let Some(contract_bytecode) = contract_bytecode {
        println!("{:#?}", contract_bytecode);
    } else {
        println!("0x");
    }
    Ok(())
}
