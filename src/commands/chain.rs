use crate::{
    config::ZKSyncConfig,
    utils::{
        balance::{display_l1_balance, display_l2_balance},
        chain::{display_batches_details, display_batches_proof_time_from_l1_batch_details},
        try_l1_provider_from_config, try_l2_provider_from_config,
    },
};
use clap::Subcommand;
use eyre::ContextCompat;
use std::collections::HashMap;
use zksync_ethers_rs::{
    abi::Hash,
    core::utils::format_ether,
    providers::Middleware,
    types::{zksync::L1BatchNumber, Address, Bytes, U64},
    ZKMiddleware,
};

#[derive(Subcommand)]
pub(crate) enum Command {
    #[clap(about = "Get the deployed bytecode of a contract")]
    GetCode { contract: Address },
    #[clap(about = "Get a transaction by hash")]
    GetTransaction { transaction: Hash },
    #[clap(about = "Retrieves the addresses of canonical bridge contracts for ZKsync Era.")]
    BridgeContracts,
    #[clap(about = "Retrieves the bytecode of a transaction by its hash.")]
    GetBytecodeByHash { hash: Hash },
    #[clap(
        about = "Lists confirmed tokens. Confirmed in the method name means any token bridged to ZKsync Era via the official bridge."
    )]
    ConfirmedTokens {
        #[clap(long, name = "FROM")]
        from: u32,
        #[clap(long, name = "LIMIT")]
        limit: u8,
    },
    #[clap(about = "Retrieves details for a given L1 batch.")]
    L1BatchDetails {
        #[clap(short = 'n', num_args = 1..)]
        batches: Vec<L1BatchNumber>,
        #[clap(
            short = 't',
            default_value_t = false,
            help = "The command displays the proof-time based on the L1 txs' timestamps."
        )]
        proof_time: bool,
    },
    L2ToL1LogProof {
        #[clap(long, name = "TRANSACTION_HASH")]
        transaction: Hash,
        #[clap(long, name = "L2_TO_L1_LOG_INDEX")]
        log_index: Option<u64>,
        #[clap(
            long,
            action,
            conflicts_with_all = ["MESSAGE_PROOF", "MESSAGE_BLOCK", "MESSAGE_SENDER", "msg"],
            group = "log",
            name = "LOG_PROOF"
        )]
        log_proof: bool,
        #[clap(
            long,
            action,
            conflicts_with_all = ["LOG_PROOF", "L2_TO_L1_LOG_INDEX"],
            group = "msg",
            name = "MESSAGE_PROOF"
        )]
        msg_proof: bool,
        #[clap(
            long,
            conflicts_with_all = ["LOG_PROOF", "L2_TO_L1_LOG_INDEX"],
            group = "msg",
            name = "MESSAGE_BLOCK"
        )]
        block: U64,
        #[clap(
            long,
            conflicts_with_all = ["LOG_PROOF", "L2_TO_L1_LOG_INDEX"],
            group = "msg",
            name = "MESSAGE_SENDER"
        )]
        sender: Address,
        #[clap(
            long,
            conflicts_with_all = ["LOG_PROOF", "L2_TO_L1_LOG_INDEX"],
            group = "msg",
            name = "MESSAGE"
        )]
        msg: Hash,
    },
    #[clap(about = "Retrieves the main contract address.", visible_aliases = &["main", "hyperchain", "diamond"])]
    MainContract {
        #[arg(long, short = 'e', default_value_t = false)]
        explorer_url: bool,
    },
    #[clap(
        about = "Retrieves the bridge hub contract address.",
        visible_alias = "bridgehub"
    )]
    BridgehubContract {
        #[arg(long, short = 'e', default_value_t = false)]
        explorer_url: bool,
    },
    #[clap(
        about = "Retrieves the testnet paymaster address, specifically for interactions within the ZKsync Sepolia Testnet environment. Note: This method is only applicable for ZKsync Sepolia Testnet."
    )]
    TestnetPaymaster {
        #[arg(long, short = 'e', default_value_t = false)]
        explorer_url: bool,
    },
    #[clap(about = "Retrieves the L1 chain ID.")]
    L1ChainID,
    #[clap(about = "Retrieves the L1 base token address.")]
    L1BaseTokenAddress {
        #[arg(long, short = 'e', default_value_t = false)]
        explorer_url: bool,
    },
    #[clap(about = "Gets all account balances for a given address.")]
    AllAccountBalances { account_address: Address },
    #[clap(about = "Retrieves the current L1 batch number.")]
    L1BatchNumber,
    #[clap(about = "Retrieves details for a given block.")]
    BlockDetails { block_number: u32 },
    #[clap(about = "Retrieves details for a given transaction.")]
    TransactionDetails { transaction: Hash },
    #[clap(about = "Retrieves the current L1 gas price.")]
    L1GasPrice,
    #[clap(about = "Retrieves the current fee parameters.")]
    FeeParams,
    #[clap(about = "Gets the protocol version.")]
    ProtocolVersion { id: Option<u16> },
    #[clap(about = "Get the balance of an account.")]
    Balance {
        of: Address,
        #[arg(long = "token")]
        token_address: Option<Address>,
        #[arg(long = "l2", required = false)]
        l2: bool,
        #[arg(long = "l1", required = false)]
        l1: bool,
    },
    #[clap(about = "Gets the finalize deposit transaction hash.")]
    FinalizeDepositTx {
        l1_deposit_tx_hash: Hash,
        #[clap(long, short = 'e', required = false)]
        explorer_url: bool,
    },
}

impl Command {
    pub async fn run(self, cfg: ZKSyncConfig) -> eyre::Result<()> {
        let l2_provider = try_l2_provider_from_config(&cfg)?;
        let l1_provider = try_l1_provider_from_config(&cfg)?;

        let l1_explorer_url = cfg
            .clone()
            .network
            .l1_explorer_url
            .filter(|url| !url.is_empty())
            .unwrap_or("https://sepolia.etherscan.io".to_owned());

        let l2_explorer_url = cfg
            .clone()
            .network
            .l2_explorer_url
            .filter(|url| !url.is_empty())
            .unwrap_or("http://localhost:3010".to_owned());

        match self {
            Command::GetCode { contract } => {
                let deployed_bytecode = l2_provider.get_code(contract, None).await?;
                println!("{deployed_bytecode:#?}");
            }
            Command::GetTransaction { transaction } => {
                let transaction = l2_provider
                    .get_transaction(transaction)
                    .await?
                    .context("No pending transaction")?;
                println!("{transaction:#?}");
            }
            Command::BridgeContracts => {
                let bridge_contracts = l2_provider.get_bridge_contracts().await?;
                if let Some(l1_shared_bridge) = bridge_contracts.l1_shared_default_bridge {
                    println!("L1 Shared Bridge: {l1_shared_bridge:#?}");
                } else {
                    println!("L1 Shared Bridge: Not set");
                }
                if let Some(l1_erc20_bridge) = bridge_contracts.l1_erc20_default_bridge {
                    println!("L1 ERC20 Bridge: {l1_erc20_bridge:#?}");
                } else {
                    println!("L1 ERC20 Bridge: Not set");
                }
                if let Some(l1_weth_bridge) = bridge_contracts.l1_weth_bridge {
                    println!("L1 WETH Bridge: {l1_weth_bridge:#?}");
                } else {
                    println!("L1 WETH Bridge: Not set");
                }
                if let Some(l2_shared_bridge) = bridge_contracts.l2_shared_default_bridge {
                    println!("L2 Shared Bridge: {l2_shared_bridge:#?}");
                } else {
                    println!("L2 Shared Bridge: Not set");
                }
                if let Some(l2_erc20_bridge) = bridge_contracts.l2_erc20_default_bridge {
                    println!("L2 ERC20 Bridge: {l2_erc20_bridge:#?}");
                } else {
                    println!("L2 ERC20 Bridge: Not set");
                }
                if let Some(l2_weth_bridge) = bridge_contracts.l2_weth_bridge {
                    println!("L2 WETH Bridge: {l2_weth_bridge:#?}");
                } else {
                    println!("L2 WETH Bridge: Not set");
                }
            }
            Command::GetBytecodeByHash { hash } => {
                let contract_bytecode = l2_provider
                    .get_bytecode_by_hash(hash)
                    .await?
                    .map(Bytes::from);
                if let Some(contract_bytecode) = contract_bytecode {
                    println!("{contract_bytecode:#?}");
                } else {
                    println!("0x");
                }
            }
            Command::ConfirmedTokens { from, limit } => {
                let confirmed_tokens = l2_provider.get_confirmed_tokens(from, limit).await?;
                println!("Confirmed Tokens: {confirmed_tokens:#?}");
            }
            Command::L1BatchDetails {
                mut batches,
                proof_time,
            } => {
                let current_batch = l2_provider.get_l1_batch_number().await?.as_u32().into();

                if batches.is_empty() {
                    batches.push(current_batch);
                }

                if proof_time {
                    display_batches_proof_time_from_l1_batch_details(
                        batches,
                        current_batch,
                        l2_provider,
                    )
                    .await?;
                } else {
                    display_batches_details(batches, current_batch, l2_provider).await?;
                }
            }
            Command::L2ToL1LogProof {
                transaction,
                log_index,
                log_proof,
                msg_proof,
                block,
                sender,
                msg,
            } => {
                let proof = if log_proof {
                    l2_provider
                        .get_l2_to_l1_log_proof(transaction, log_index)
                        .await?
                } else if msg_proof {
                    l2_provider
                        .get_l2_to_l1_msg_proof(block, sender, msg, log_index)
                        .await?
                } else {
                    eyre::bail!("no type of proof provided")
                }
                .context("no proof");
                log::info!("{proof:#?}");
            }
            Command::MainContract { explorer_url } => {
                let main_contract_address = l2_provider.get_main_contract().await?;
                if explorer_url && cfg.network.l2_explorer_url.is_some() {
                    println!(
                        "Main Contract:\n{l2_explorer_url}/address/{main_contract_address:#?}",
                    );
                } else {
                    println!("{main_contract_address:#?}");
                }
            }
            Command::BridgehubContract { explorer_url } => {
                let bridgehub_contract_address = l2_provider.get_bridgehub_contract().await?;
                if explorer_url && cfg.network.l2_explorer_url.is_some() {
                    println!(
                        "Bridgehub Contract:\n{l2_explorer_url}/address/{bridgehub_contract_address:#?}",
                    );
                } else {
                    println!("{bridgehub_contract_address:#?}");
                }
            }
            Command::TestnetPaymaster { explorer_url } => {
                let testnet_paymaster_address = l2_provider.get_testnet_paymaster().await?;
                if explorer_url && cfg.network.l2_explorer_url.is_some() {
                    println!(
                        "Testnet Paymaster Address:\n{l2_explorer_url}/address/{testnet_paymaster_address:#?}",
                    );
                } else {
                    println!("{testnet_paymaster_address:#?}");
                }
            }
            Command::L1ChainID => {
                let l1_chain_id = l2_provider.get_l1_chain_id().await?;
                println!("L1 Chain ID: {l1_chain_id:#?}");
            }
            Command::L1BaseTokenAddress { explorer_url } => {
                let l1_base_token_address = l2_provider.get_base_token_l1_address().await?;
                if explorer_url && cfg.network.l2_explorer_url.is_some() {
                    println!(
                        "L1 Base Token Address:\n{l1_explorer_url}/address/{l1_base_token_address:#?}",
                    );
                } else {
                    println!("{l1_base_token_address:#?}");
                }
            }
            Command::AllAccountBalances { account_address } => {
                // Retrieving the L2 balances, the token addresses will not be usable on L1
                // A way to transform the token addresses from L2 to L1 may be needed
                let all_account_balances = l2_provider
                    .get_all_account_balances(account_address)
                    .await?;
                let mut all_account_parsed_balances: HashMap<Address, String> = HashMap::new();
                for (k, v) in all_account_balances {
                    // Assuming all tokens have 18 Decimals
                    // To have display the balance better use the address provided with this cmd
                    // And use the balance cmd with the token address
                    let v = format_ether(v);
                    all_account_parsed_balances.insert(k, v);
                }
                println!("{all_account_parsed_balances:#?}");
            }
            Command::L1BatchNumber => {
                let l1_batch_number = l2_provider.get_l1_batch_number().await?;
                println!("Latest L1 Batch Number: {l1_batch_number:#?}");
            }
            Command::BlockDetails { block_number } => {
                let block_details = l2_provider.get_block_details(block_number).await?;
                if let Some(block_details) = block_details {
                    println!("{block_details:#?}");
                } else {
                    println!("Block {block_number} not found");
                }
            }
            Command::TransactionDetails {
                transaction: transaction_hash,
            } => {
                let transaction_details = l2_provider
                    .get_transaction_details(transaction_hash)
                    .await?
                    .context("No pending transaction")?;
                println!("{transaction_details:#?}");
            }
            Command::L1GasPrice => {
                let current_l1_gas_price = l2_provider.get_l1_gas_price().await?;
                println!("Current L1 Gas Price (wei): {current_l1_gas_price:#?}");
            }
            Command::FeeParams => {
                let fee_params = l2_provider.get_fee_params().await?;
                println!("{fee_params:#?}");
            }
            Command::ProtocolVersion { id } => {
                let protocol_version = l2_provider.get_protocol_version(id).await?;
                if let Some(protocol_version) = protocol_version {
                    println!("{protocol_version:#?}");
                } else {
                    println!("Protocol version not found");
                }
            }
            Command::Balance {
                of,
                token_address,
                l2,
                l1,
            } => {
                if l2 || !l1 {
                    let base_token_address = l2_provider.get_base_token_l1_address().await?;
                    display_l2_balance(
                        of,
                        token_address,
                        &l1_provider,
                        &l2_provider,
                        base_token_address,
                        l1,
                    )
                    .await?;
                };
                if l1 {
                    display_l1_balance(of, token_address, &l1_provider).await?;
                };
            }
            Command::FinalizeDepositTx {
                l1_deposit_tx_hash,
                explorer_url,
            } => {
                let deposit_finalization_hash =
                    zksync_ethers_rs::deposit::l2_deposit_tx_hash(l1_deposit_tx_hash, &l1_provider)
                        .await;
                if explorer_url {
                    let url = cfg
                        .network
                        .l2_explorer_url
                        .context("L2 Explorer URL missing in config")?;
                    println!("Deposit finalization: {url}/tx/{deposit_finalization_hash:#?}");
                } else {
                    println!("Deposit finalization hash: {deposit_finalization_hash:#?}");
                }
            }
        };
        Ok(())
    }
}
