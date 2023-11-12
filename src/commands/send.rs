use crate::cli::ZKSyncConfig;
use clap::Args as ClapArgs;
use eyre::ContextCompat;
use zksync_web3_rs::eip712::Eip712TransactionRequest;
use zksync_web3_rs::prelude::abi::{encode, HumanReadableParser};
use zksync_web3_rs::providers::Middleware;
use zksync_web3_rs::signers::{LocalWallet, Signer};
use zksync_web3_rs::types::Bytes;
use zksync_web3_rs::zks_provider::ZKSProvider;
use zksync_web3_rs::zks_utils;
use zksync_web3_rs::{providers::Provider, types::Address};

// TODO: Optional parameters were omitted, they should be added in the future.
#[derive(ClapArgs)]
pub(crate) struct Args {
    #[clap(short, long, name = "CONTRACT_ADDRESS")]
    pub contract: Address,
    #[clap(short, long, name = "FUNCTION_SIGNATURE", conflicts_with = "data")]
    pub function: Option<String>,
    #[clap(short, long, num_args(1..), name = "FUNCTION_ARGS", conflicts_with = "data")]
    pub args: Option<Vec<String>>,
    #[clap(long, name = "DATA")]
    pub data: Option<Bytes>,
    #[clap(long, num_args(1..), requires = "data", name = "OUTPUT_TYPES")]
    pub output_types: Option<Vec<String>>,
    #[clap(short, long, name = "PRIVATE_KEY")]
    pub private_key: LocalWallet,
    #[clap(long, name = "CHAIN_ID")]
    pub chain_id: u16,
}

pub(crate) async fn run(args: Args, config: ZKSyncConfig) -> eyre::Result<()> {
    let provider = if let Some(port) = config.l2_port {
        Provider::try_from(format!(
            "http://{host}:{port}",
            host = config.host,
            port = port
        ))?
    } else {
        Provider::try_from(format!("{host}", host = config.host,))?
    }
    .interval(std::time::Duration::from_millis(10));

    let sender = args.private_key.with_chain_id(args.chain_id);

    let mut request = Eip712TransactionRequest::new()
        .r#type(zks_utils::EIP712_TX_TYPE)
        .to(args.contract);

    let func: String;
    if let Some(data) = args.data {
        request = request.data(data);
    } else if let Some(function_args) = args.args {
        // Note: CLI syntactic sugar need to be handle in the run() function.
        // If more sugar cases are needed, we should switch to a match statement.
        let function_signature = if args.function.clone().unwrap().is_empty() {
            "function()"
        } else {
            func = args.function.unwrap();
            &func
        };

        let function = if args.contract == zks_utils::ECADD_PRECOMPILE_ADDRESS {
            zks_utils::ec_add_function()
        } else if args.contract == zks_utils::ECMUL_PRECOMPILE_ADDRESS {
            zks_utils::ec_mul_function()
        } else if args.contract == zks_utils::MODEXP_PRECOMPILE_ADDRESS {
            zks_utils::mod_exp_function()
        } else {
            HumanReadableParser::parse_function(function_signature)?
        };
        let function_args =
            function.decode_input(&zks_utils::encode_args(&function, &function_args)?)?;

        let data = match (
            !function_args.is_empty(),
            zks_utils::is_precompile(args.contract),
        ) {
            // The contract to call is a precompile with arguments.
            (true, true) => encode(&function_args),
            // The contract to call is a regular contract with arguments.
            (true, false) => function.encode_input(&function_args)?,
            // The contract to call is a precompile without arguments.
            (false, true) => Default::default(),
            // The contract to call is a regular contract without arguments.
            (false, false) => function.short_signature().into(),
        };

        request = request.data(data);
    }

    let tx_receipt = provider
        .send_transaction_eip712(&sender, request.clone())
        .await?
        .await?
        .context("Failed to get transaction receipt")?;

    request = request
        .from(sender.address())
        .chain_id(sender.chain_id())
        .nonce(provider.get_transaction_count(sender.address(), None).await?)
        .gas_price(provider.get_gas_price().await?)
        .max_fee_per_gas(provider.get_gas_price().await?);

    log::info!("Estimated Gas: {:?}", provider.estimate_fee(request).await?);
    log::info!("{:?}", tx_receipt.transaction_hash);

    Ok(())
}
