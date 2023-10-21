use crate::cli::ZKSyncConfig;
use crate::commands::compile;
use clap::Args as ClapArgs;
use eyre::ContextCompat;
use zksync_web3_rs::prelude::abi::Token;
use zksync_web3_rs::signers::LocalWallet;
use zksync_web3_rs::types::Bytes;
use zksync_web3_rs::zks_wallet::DeployRequest;
use zksync_web3_rs::ZKSWallet;
use zksync_web3_rs::{providers::Provider, signers::Signer};

#[derive(ClapArgs)]
pub(crate) struct Args {
    #[clap(
        long,
        name = "CONTRACT PATH",
        requires = "contract_name",
        conflicts_with = "bytecode"
    )]
    pub contract: Option<String>,
    #[clap(
        long,
        name = "CONTRACT NAME",
        requires = "contract",
        conflicts_with = "bytecode"
    )]
    pub contract_name: Option<String>,
    #[clap(long, num_args(1..), name = "CONSTRUCTOR_ARGS")]
    constructor_args: Vec<String>,
    #[clap(short, long, name = "PRIVATE KEY")]
    pub private_key: LocalWallet,
    #[clap(long, name = "CONTRACT BYTECODE")]
    pub bytecode: Option<Bytes>,
    #[clap(long, name = "CHAIN_ID")]
    pub chain_id: u16,
}

pub(crate) async fn run(args: Args, config: ZKSyncConfig) -> eyre::Result<()> {
    let era_provider = Provider::try_from(format!(
        "http://{host}:{port}",
        host = config.host,
        port = config.port
    ))?;
    let wallet = args.private_key.with_chain_id(args.chain_id);
    let zk_wallet = ZKSWallet::new(wallet, None, Some(era_provider.clone()), None)?;
    let contract_address = if let Some(bytecode) = args.bytecode {
        zk_wallet
            .deploy_from_bytecode(&bytecode, None, None::<Token>)
            .await?
    } else if let Some(contract_path) = args.contract.clone() {
        let artifact = compile::compiler::compile(
            &contract_path,
            &args.contract_name.context("no contract name provided")?,
            compile::compiler::Compiler::ZKSolc,
        )?;

        let compiled_bytecode = artifact.bin()?;
        let compiled_abi = artifact.abi()?;

        let deploy_request = DeployRequest::with(
            compiled_abi,
            compiled_bytecode.to_vec(),
            args.constructor_args,
        );

        // TODO(Ivan): Wait until deploy does not compile anymore.
        zk_wallet.deploy(&deploy_request).await?
    } else {
        panic!("no bytecode or contract path provided")
    };
    log::info!("{contract_address:#?}");
    Ok(())
}
