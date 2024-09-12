use crate::{config::ZKSyncConfig, utils::contract::*, utils::wallet::get_wallet_l1_l2_providers};
use clap::Subcommand;
use eyre::ContextCompat;
use spinoff::{spinners, Color, Spinner};
use std::{ops::Div, str::FromStr};
use zksync_ethers_rs::{
    abi::Abi,
    contract::ContractFactory,
    eip712::{DeployRequest, Eip712TransactionRequest},
    providers::Middleware,
    types::{transaction::eip2718::TypedTransaction, Bytes, Eip1559TransactionRequest, H160, U256},
    ZKMiddleware,
};

#[derive(Subcommand)]
pub(crate) enum Command {
    #[clap(about = "Call view functions on a contract.")]
    Call {
        #[clap(long = "contract_address", short = 'c', required = true)]
        contract_address: String,
        #[clap(long = "function_signature", short = 'f', required = true)]
        function_signature: String,
        #[clap(long = "args", short = 'a')]
        args: Option<Vec<String>>,
        #[clap(long = "l1", default_value_t = false)]
        l1: bool,
    },
    #[clap(about = "Deploy a contract.")]
    Deploy {
        #[clap(long = "bytecode", short = 'b', required = true)]
        bytecode: String,
        #[clap(long = "constructor_args", short = 'a')]
        constructor_args: Option<Vec<String>>,
        #[clap(long = "constructor_types", short = 't')]
        constructor_types: Option<Vec<String>>,
        #[clap(long = "l1", default_value_t = false)]
        l1: bool,
    },
    #[clap(about = "Call non-view functions on a contract.")]
    Send {
        #[clap(long = "contract_address", short = 'c', required = true)]
        contract_address: String,
        #[clap(long = "function_signature", short = 'f', required = true)]
        function_signature: String,
        #[clap(long = "args", short = 'a')]
        args: Option<Vec<String>>,
        #[clap(long = "l1", default_value_t = false)]
        l1: bool,
    },
}

impl Command {
    pub async fn run(self, cfg: ZKSyncConfig) -> eyre::Result<()> {
        let (zk_wallet, _l1_provider, _l2_provider) = get_wallet_l1_l2_providers(cfg)?;

        match self {
            Command::Call {
                contract_address,
                function_signature,
                args,
                l1,
            } => {
                let selector = get_fn_selector(&function_signature);
                let types = parse_signature(&function_signature)?;
                let tx_data = encode_call(Some(selector), None, args, types)?;
                let tx_data_bytes = Bytes::from(tx_data);

                let mut tx = Eip1559TransactionRequest::new()
                    .to(H160::from_str(&contract_address)?)
                    .data(tx_data_bytes)
                    .value(U256::zero());

                let receipt = if l1 {
                    tx = tx.from(zk_wallet.l1_address());
                    zk_wallet.l1_provider().call(&tx.into(), None).await?
                } else {
                    tx = tx.from(zk_wallet.l2_address());
                    zk_wallet.l2_provider().call(&tx.into(), None).await?
                };

                println!("{receipt}");
            }
            Command::Deploy {
                bytecode,
                constructor_args,
                constructor_types,
                l1,
            } => {
                let bytecode_vec = hex::decode(
                    bytecode
                        .strip_prefix("0x")
                        .context("Bytecode without 0x prefix")?,
                )?
                .to_vec();

                let tx_data = encode_call(
                    None,
                    Some(bytecode_vec.clone()),
                    constructor_args.clone(),
                    constructor_types,
                )?;
                let tx_data_bytes = Bytes::from(tx_data);

                let mut spinner =
                    Spinner::new(spinners::Dots, "Deploying Contract...", Color::Blue);
                if l1 {
                    let factory = ContractFactory::new(
                        Abi::default(), // Don't care
                        tx_data_bytes,
                        zk_wallet.l1_signer(),
                    );
                    let contract = factory.deploy(())?.send().await?;
                    let msg = format!("Contract deployed at: {:?}", contract.address());
                    spinner.success(&msg);
                } else {
                    let deploy_request: DeployRequest = DeployRequest::with(
                        Abi::default(),
                        bytecode_vec,
                        constructor_args.unwrap_or(vec![]),
                    );

                    let transaction: Eip712TransactionRequest = deploy_request.try_into()?;

                    let receipt = zk_wallet.send_transaction_eip712(transaction).await;
                    let msg = format!("Contract deployed at: {:?}", receipt.contract_address);
                    spinner.success(&msg);
                }
            }
            Command::Send {
                contract_address,
                function_signature,
                args,
                l1,
            } => {
                let selector = get_fn_selector(&function_signature);
                let types = parse_signature(&function_signature)?;
                let tx_data = encode_call(Some(selector), None, args, types)?;
                let tx_data_bytes = Bytes::from(tx_data);

                let mut raw_tx = Eip1559TransactionRequest::new()
                    .to(H160::from_str(&contract_address)?)
                    .data(tx_data_bytes)
                    .value(U256::zero());

                let (signer, fees) = if l1 {
                    let signer = zk_wallet.l1_signer();

                    raw_tx = raw_tx
                        .from(zk_wallet.l1_address())
                        .chain_id(zk_wallet.l1_provider().get_chainid().await?.as_u64());

                    let tx: TypedTransaction = raw_tx.clone().into();
                    let fees = zk_wallet.l1_provider().estimate_fee(&tx).await?;
                    (signer, fees)
                } else {
                    let signer = zk_wallet.l2_signer();

                    raw_tx = raw_tx
                        .from(zk_wallet.l2_address())
                        .chain_id(zk_wallet.l2_provider().get_chainid().await?.as_u64());

                    let tx: TypedTransaction = raw_tx.clone().into();
                    let fees = zk_wallet.l2_provider().estimate_fee(&tx).await?;
                    (signer, fees)
                };

                raw_tx = raw_tx
                    .max_fee_per_gas(fees.max_fee_per_gas)
                    .max_priority_fee_per_gas(fees.max_priority_fee_per_gas)
                    .gas(fees.gas_limit.div(10_i32).saturating_mul(11_i32.into())); // headroom 10% extra

                let tx: TypedTransaction = raw_tx.into();
                let receipt = signer
                    .send_transaction(tx, None)
                    .await?
                    .await?
                    .context("error unwrapping")?;
                println!("{receipt:#?}");
            }
        };
        Ok(())
    }
}
