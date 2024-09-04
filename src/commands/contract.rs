use crate::{config::ZKSyncConfig, utils::wallet::get_wallet_l1_l2_providers};
use clap::Subcommand;
use std::str::FromStr;
use zksync_ethers_rs::{
    abi::Token,
    core::utils::keccak256,
    providers::Middleware,
    types::{
        transaction::eip2718::TypedTransaction, Address, Bytes, Eip1559TransactionRequest, H160,
        U256,
    },
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
        constructor_args: Vec<String>,
    },
    #[clap(about = "Call non-view functions on a contract.")]
    Send {
        #[clap(long = "contract_address", short = 'c', required = true)]
        contract_address: String,
        #[clap(long = "function_signature", short = 'f', required = true)]
        function_signature: String,
        #[clap(long = "args", short = 'a')]
        args: Option<Vec<String>>,
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
                let tx_data = encode_function_call(selector, args, types)?;
                let tx_data_bytes = Bytes::from(tx_data);

                let tx = TypedTransaction::Eip1559(
                    Eip1559TransactionRequest::new()
                        .from(zk_wallet.l1_address())
                        .to(H160::from_str(&contract_address)?)
                        .data(tx_data_bytes)
                        .value(U256::zero()),
                );

                let receipt = if l1 {
                    zk_wallet.l1_provider().call(&tx, None).await?
                } else {
                    zk_wallet.l2_provider().call(&tx, None).await?
                };

                println!("{receipt}");
            }
            Command::Deploy {
                bytecode: _,
                constructor_args: _,
            } => todo!("Deploy"),
            Command::Send {
                contract_address: _,
                function_signature: _,
                args: _,
            } => todo!(),
        };
        Ok(())
    }
}

fn encode_function_call(
    selector: [u8; 4],
    args: Option<Vec<String>>,
    types: Option<Vec<&str>>,
) -> eyre::Result<Vec<u8>> {
    let mut data = Vec::new();
    data.extend_from_slice(&selector);

    let tokens = match (args, types) {
        (Some(a), Some(t)) => {
            if a.len() != t.len() {
                return Err(eyre::eyre!("Argument count does not match type count"));
            }
            a.into_iter()
                .zip(t.into_iter())
                .map(|(arg, arg_type)| {
                    match arg_type {
                        "address" => {
                            // Parse as Address
                            let address: Address = arg
                                .parse()
                                .map_err(|_e| eyre::eyre!("Invalid address format"))?;
                            Ok(Token::Address(address))
                        }
                        "uint256" => {
                            // Parse as uint256
                            let value: u64 = arg
                                .parse()
                                .map_err(|_e| eyre::eyre!("Invalid uint256 format"))?;
                            Ok(Token::Uint(U256::from(value)))
                        }
                        "string" => {
                            // Parse as string
                            Ok(Token::String(arg))
                        }
                        _ => Err(eyre::eyre!("Unsupported argument type")),
                    }
                })
                .collect::<Result<Vec<Token>, eyre::Report>>()?
        }
        (Some(_), None) => {
            return Err(eyre::eyre!("Types not provided"));
        }
        (None, _) => vec![],
    };

    let encoded_args = zksync_ethers_rs::abi::encode(&tokens);
    data.extend(encoded_args);
    Ok(data)
}

fn parse_signature(signature: &str) -> eyre::Result<Option<Vec<&str>>> {
    let mut types = Vec::new();

    if let Some(start) = signature.find('(') {
        if let Some(end) = signature.rfind(')') {
            let params = &signature[start + 1..end];

            // Split the parameters by comma
            for param in params.split(',') {
                let trimmed_param = param.trim();
                types.push(trimmed_param)
            }
        } else {
            return Err(eyre::eyre!("Missing closing parenthesis in signature"));
        }
    } else {
        return Err(eyre::eyre!("Missing opening parenthesis in signature"));
    }
    if types.is_empty() {
        return Ok(None);
    }
    Ok(Some(types))
}

fn get_fn_selector(function_signature: &str) -> [u8; 4] {
    let hash = keccak256(function_signature.as_bytes());
    let mut selector = [0_u8; 4];
    selector.copy_from_slice(&hash[0..4]);
    selector
}
