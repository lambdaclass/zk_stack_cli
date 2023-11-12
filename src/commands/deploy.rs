use crate::cli::ZKSyncConfig;
use crate::commands::compile;
use crate::commands::compile::output::ZKSArtifact;
use clap::Args as ClapArgs;
use eyre::ContextCompat;
use zksync_web3_rs::signers::LocalWallet;
use zksync_web3_rs::zks_wallet::DeployRequest;
use zksync_web3_rs::ZKSWallet;
use zksync_web3_rs::{providers::Provider, signers::Signer};

#[derive(ClapArgs)]
pub(crate) struct Args {
    #[clap(short, long, name = "PROJECT_ROOT_PATH")]
    pub project_root: String,
    #[clap(
        long,
        name = "CONTRACT PATH",
        requires = "contract_name",
        conflicts_with = "contract_artifact"
    )]
    pub contract: Option<String>,
    #[clap(
        long,
        name = "CONTRACT NAME",
        requires = "contract",
        conflicts_with = "contract_artifact"
    )]
    pub contract_name: Option<String>,
    #[clap(
        long,
        name = "CONTRACT NAME",
        requires = "contract",
        conflicts_with = "contract, contract_name"
    )]
    pub contract_artifact: Option<String>,
    #[clap(long, num_args(1..), name = "CONSTRUCTOR_ARGS")]
    constructor_args: Vec<String>,
    #[clap(short, long, name = "PRIVATE KEY")]
    pub private_key: LocalWallet,
    #[clap(long, name = "CHAIN_ID")]
    pub chain_id: u16,
}

pub(crate) async fn run(args: Args, config: ZKSyncConfig) -> eyre::Result<()> {
    let era_provider = Provider::try_from(format!(
        "http://{host}:{port}",
        host = config.host,
        port = config.l2_port
    ))?;
    let wallet = args.private_key.with_chain_id(args.chain_id);
    let zk_wallet = ZKSWallet::new(wallet, None, Some(era_provider.clone()), None)?;
    let contract_address = if let Some(contract_path) = args.contract {
        let artifact = compile::compiler::compile(
            &args.project_root,
            &contract_path,
            &args.contract_name.context("Contract name is required")?,
            compile::compiler::Compiler::ZKSolc,
        )?;

        let compiled_bytecode = artifact.bin()?;
        let compiled_abi = artifact.abi()?;

        let deploy_request = DeployRequest::with(
            compiled_abi,
            compiled_bytecode.to_vec(),
            args.constructor_args,
        );

        zk_wallet.deploy(&deploy_request).await?
    } else if let Some(artifact_path) = args.contract_artifact {
        let artifact: ZKSArtifact = serde_json::from_slice(&std::fs::read(artifact_path)?)?;
        let compiled_bytecode = artifact.bin.context("Contract bytecode is missing")?;
        let compiled_abi = artifact.abi.context("Contract ABI is missing")?;

        let deploy_request = DeployRequest::with(
            compiled_abi,
            compiled_bytecode.to_vec(),
            args.constructor_args,
        );

        zk_wallet.deploy(&deploy_request).await?
    } else {
        return Err(eyre::eyre!("Contract path or artifact path is required"));
    };
    log::info!("{contract_address:#?}");
    Ok(())
}
