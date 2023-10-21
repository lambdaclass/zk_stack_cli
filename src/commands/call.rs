use crate::cli::ZKSyncConfig;
use clap::Args as ClapArgs;
use eyre::eyre;
use zksync_web3_rs::prelude::abi::{encode, decode, ParamType, Tokenize, HumanReadableParser};
use zksync_web3_rs::providers::Middleware;
use zksync_web3_rs::signers::LocalWallet;
use zksync_web3_rs::types::transaction::eip2718::TypedTransaction;
use zksync_web3_rs::types::{Bytes, Eip1559TransactionRequest};
use zksync_web3_rs::zks_utils;
use zksync_web3_rs::{providers::Provider, types::Address};

// TODO: Optional parameters were omitted, they should be added in the future.
#[derive(ClapArgs)]
pub(crate) struct Args {
    #[clap(short, long, name = "CONTRACT_ADDRESS")]
    pub contract: Address,
    #[clap(short, long, name = "FUNCTION_SIGNATURE")]
    pub function: String,
    #[clap(short, long, num_args(1..), name = "FUNCTION_ARGS")]
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
    let provider = Provider::try_from(format!(
        "http://{host}:{port}",
        host = config.host,
        port = config.port
    ))?;

    // Note: CLI syntactic sugar need to be handle in the run() function.
    // If more sugar cases are needed, we should switch to a match statement.
    let function_signature = if args.function.is_empty() {
        "function()"
    } else {
        &args.function
    };

    let mut request = Eip1559TransactionRequest::new()
        .to(args.contract)
        .chain_id(args.chain_id);

    if let Some(data) = args.data {
        request = request.data(data);
    } else if let Some(function_args) = args.args {
        let function = if args.contract == zks_utils::ECADD_PRECOMPILE_ADDRESS {
            zks_utils::ec_add_function()
        } else if args.contract == zks_utils::ECMUL_PRECOMPILE_ADDRESS {
            zks_utils::ec_mul_function()
        } else if args.contract == zks_utils::MODEXP_PRECOMPILE_ADDRESS {
            zks_utils::mod_exp_function()
        } else {
            HumanReadableParser::parse_function(&function_signature)?
        };
        let function_args = function.decode_input(&zks_utils::encode_args(&function, &function_args)?)?;

        let data = match (!function_args.is_empty(), zks_utils::is_precompile(args.contract)) {
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

    let transaction: TypedTransaction = request.into();

    let encoded_output = Middleware::call(&provider, &transaction, None).await?;

    let decoded_output = if let Some(output_types) = args.output_types {
        let parsed_param_types: Vec<ParamType> = output_types
            .iter()
            .map(|output_type| match output_type.as_str() {
                "uint256" => Ok(ParamType::Uint(256)),
                "sint256" => Ok(ParamType::Int(256)),
                other => Err(eyre!("Unable to parse output type: {other}")),
            })
            .collect::<eyre::Result<Vec<ParamType>>>()?;
        decode(&parsed_param_types, &encoded_output)?
    } else {
        encoded_output.into_tokens()
    };

    log::info!("{decoded_output:?}");
    Ok(())
}
