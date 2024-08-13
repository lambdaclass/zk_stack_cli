use crate::{
    commands::{
        get_all_account_balances, get_block_details, get_bridge_contracts, get_bridgehub_contract,
        get_bytecode_by_hash, get_code, get_confirmed_tokens, get_fee_params,
        get_l1_base_token_address, get_l1_batch_details, get_l1_batch_number, get_l1_chain_id,
        get_l1_gas_price, get_l2_to_l1_proof, get_main_contract, get_protocol_version,
        get_transaction, testnet_paymaster, transaction_details,
    },
    config::ZKSyncConfig,
};
use clap::{command, Parser, Subcommand};

pub const VERSION_STRING: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser)]
#[command(name="zk", author, version=VERSION_STRING, about, long_about = None)]
struct ZKSyncCLI {
    #[command(subcommand)]
    command: ZKSyncCommand,
}

#[derive(Subcommand)]
enum ZKSyncCommand {
    #[clap(about = "Get the deployed bytecode of a contract")]
    GetCode(get_code::Args),
    #[clap(about = "Get a transaction by hash")]
    GetTransaction(get_transaction::Args),
    #[clap(about = "Retrieves the addresses of canonical bridge contracts for ZKsync Era.")]
    BridgeContracts,
    #[clap(about = "Retrieves the bytecode of a transaction by its hash.")]
    GetBytecodeByHash(get_bytecode_by_hash::Args),
    #[clap(
        about = "Lists confirmed tokens. Confirmed in the method name means any token bridged to ZKsync Era via the official bridge."
    )]
    ConfirmedTokens(get_confirmed_tokens::Args),
    #[clap(about = "Retrieves details for a given L1 batch.")]
    L1BatchDetails(get_l1_batch_details::Args),
    L2ToL1LogProof(get_l2_to_l1_proof::Args),
    #[clap(about = "Retrieves the main contract address.")]
    MainContract(get_main_contract::Args),
    #[clap(about = "Retrieves the bridge hub contract address.")]
    BridgehubContract(get_bridgehub_contract::Args),
    #[clap(
        about = "Retrieves the testnet paymaster address, specifically for interactions within the ZKsync Sepolia Testnet environment. Note: This method is only applicable for ZKsync Sepolia Testnet."
    )]
    TestnetPaymaster(testnet_paymaster::Args),
    #[clap(about = "Retrieves the L1 chain ID.")]
    L1ChainID,
    #[clap(about = "Retrieves the L1 base token address.")]
    L1BaseTokenAddress(get_l1_base_token_address::Args),
    #[clap(about = "Gets all account balances for a given address.")]
    AllAccountBalances(get_all_account_balances::Args),
    #[clap(about = "Retrieves the current L1 batch number.")]
    L1BatchNumber,
    #[clap(about = "Retrieves details for a given block.")]
    BlockDetails(get_block_details::Args),
    #[clap(about = "Retrieves details for a given transaction.")]
    TransactionDetails(transaction_details::Args),
    #[clap(about = "Retrieves the current L1 gas price.")]
    L1GasPrice,
    #[clap(about = "Retrieves the current fee parameters.")]
    FeeParams,
    #[clap(about = "Gets the protocol version.")]
    ProtocolVersion(get_protocol_version::Args),
}

pub async fn start(config: ZKSyncConfig) -> eyre::Result<()> {
    let ZKSyncCLI { command } = ZKSyncCLI::parse();
    match command {
        ZKSyncCommand::GetCode(args) => get_code::run(args, config).await?,
        ZKSyncCommand::GetTransaction(args) => get_transaction::run(args, config).await?,
        ZKSyncCommand::BridgeContracts => get_bridge_contracts::run(config).await?,
        ZKSyncCommand::GetBytecodeByHash(args) => get_bytecode_by_hash::run(args, config).await?,
        ZKSyncCommand::ConfirmedTokens(args) => get_confirmed_tokens::run(args, config).await?,
        ZKSyncCommand::L1BatchDetails(args) => get_l1_batch_details::run(args, config).await?,
        ZKSyncCommand::L2ToL1LogProof(args) => get_l2_to_l1_proof::run(args, config).await?,
        ZKSyncCommand::MainContract(args) => get_main_contract::run(args, config).await?,
        ZKSyncCommand::BridgehubContract(args) => get_bridgehub_contract::run(args, config).await?,
        ZKSyncCommand::TestnetPaymaster(args) => testnet_paymaster::run(args, config).await?,
        ZKSyncCommand::L1ChainID => get_l1_chain_id::run(config).await?,
        ZKSyncCommand::L1BaseTokenAddress(args) => {
            get_l1_base_token_address::run(args, config).await?
        }
        ZKSyncCommand::AllAccountBalances(args) => {
            get_all_account_balances::run(args, config).await?
        }
        ZKSyncCommand::L1BatchNumber => get_l1_batch_number::run(config).await?,
        ZKSyncCommand::BlockDetails(args) => get_block_details::run(args, config).await?,
        ZKSyncCommand::TransactionDetails(args) => transaction_details::run(args, config).await?,
        ZKSyncCommand::L1GasPrice => get_l1_gas_price::run(config).await?,
        ZKSyncCommand::FeeParams => get_fee_params::run(config).await?,
        ZKSyncCommand::ProtocolVersion(args) => get_protocol_version::run(args, config).await?,
    };

    Ok(())
}
