use std::str::FromStr;

use crate::cli::ZKSyncConfig;
use crate::commands::compile::project::ZKSProject;
use clap::Args as ClapArgs;
use eyre::eyre;
use eyre::ContextCompat;
use zksync_web3_rs::prelude::abi::Token;
use zksync_web3_rs::signers::LocalWallet;
use zksync_web3_rs::solc::info::ContractInfo;
use zksync_web3_rs::solc::{Project, ProjectPathsConfig};
use zksync_web3_rs::types::Bytes;
use zksync_web3_rs::zks_utils::ERA_CHAIN_ID;
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
}

pub(crate) async fn run(args: Args, config: ZKSyncConfig) -> eyre::Result<()> {
    let era_provider = Provider::try_from(format!(
        "http://{host}:{port}",
        host = config.host,
        port = config.port
    ))?;
    let wallet = args.private_key.with_chain_id(ERA_CHAIN_ID);
    let zk_wallet = ZKSWallet::new(wallet, None, Some(era_provider.clone()), None)?;
    let contract_address = if let Some(bytecode) = args.bytecode {
        zk_wallet
            .deploy_from_bytecode(&bytecode, None, None::<Token>)
            .await?
    } else if let Some(contract_path) = args.contract.clone() {
        let project = ZKSProject::from(
            Project::builder()
                .paths(ProjectPathsConfig::builder().build_with_root(contract_path.clone()))
                .set_auto_detect(true)
                .build()?,
        );

        let compilation_output = project.compile()?;
        let artifact = compilation_output
            .find_contract(ContractInfo::from_str(&format!(
                "{contract_path}:{contract_name}",
                contract_name = args.contract_name.context("no contract name provided")?,
            ))?)
            .ok_or(eyre!("Artifact not found"))?;
        let compiled_bytecode = artifact.bin.clone().context("no bytecode")?;
        let compiled_abi = artifact.abi.clone().context("no abi")?;

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
