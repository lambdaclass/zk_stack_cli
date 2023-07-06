use std::str::FromStr;

use crate::cli::ZKSyncWeb3Config;
use crate::commands::compile::ZKSProject;
use clap::Args;
use eyre::ContextCompat;
use zksync_web3_rs::abi::Token;
use zksync_web3_rs::signers::LocalWallet;
use zksync_web3_rs::solc::info::ContractInfo;
use zksync_web3_rs::solc::{Project, ProjectPathsConfig};
use zksync_web3_rs::types::Bytes;
use zksync_web3_rs::zks_utils::ERA_CHAIN_ID;
use zksync_web3_rs::ZKSWallet;
use zksync_web3_rs::{providers::Provider, signers::Signer};

#[derive(Args)]
pub(crate) struct Deploy {
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

pub(crate) async fn run(args: Deploy, config: ZKSyncWeb3Config) -> eyre::Result<()> {
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
            .find_contract(
                ContractInfo::from_str(&format!(
                    "{contract_path}:{contract_name}",
                    contract_name = args.contract_name.context("no contract name provided")?,
                ))
                .unwrap(),
            )
            .unwrap();
        let compiled_bytecode = artifact.bin.clone().context("no bytecode")?;
        let compiled_abi = artifact.abi.clone().context("no abi")?;

        // TODO(Ivan): Wait until deploy does not compile anymore.
        zk_wallet
            .deploy(
                compiled_abi,
                compiled_bytecode.to_vec(),
                None,
                Some(args.constructor_args),
            )
            .await?
            .contract_address
            .context("no contract address")?
    } else {
        panic!("no bytecode or contract path provided")
    };
    log::info!("{contract_address:#?}");
    Ok(())
}
