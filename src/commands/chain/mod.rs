use crate::config::ZKSyncConfig;
use clap::Subcommand;

mod balance;
mod finalize_deposit_transaction;
mod get_all_account_balances;
mod get_block_details;
mod get_bridge_contracts;
mod get_bridgehub_contract;
mod get_bytecode_by_hash;
mod get_code;
mod get_confirmed_tokens;
mod get_fee_params;
mod get_l1_base_token_address;
mod get_l1_batch_details;
mod get_l1_batch_number;
mod get_l1_chain_id;
mod get_l1_gas_price;
mod get_l2_to_l1_proof;
mod get_main_contract;
mod get_protocol_version;
mod get_transaction;
mod testnet_paymaster;
mod transaction_details;

#[derive(Subcommand, PartialEq)]
pub(crate) enum Command {
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
    #[clap(about = "Get the balance of an account.")]
    Balance(balance::Args),
    #[clap(about = "Gets the finalize deposit transaction hash.")]
    FinalizeDepositTx(finalize_deposit_transaction::Args),
}

pub(crate) async fn start(command: Command, cfg: ZKSyncConfig) -> eyre::Result<()> {
    match command {
        Command::GetCode(args) => get_code::run(args, cfg).await?,
        Command::GetTransaction(args) => get_transaction::run(args, cfg).await?,
        Command::BridgeContracts => get_bridge_contracts::run(cfg).await?,
        Command::GetBytecodeByHash(args) => get_bytecode_by_hash::run(args, cfg).await?,
        Command::ConfirmedTokens(args) => get_confirmed_tokens::run(args, cfg).await?,
        Command::L1BatchDetails(args) => get_l1_batch_details::run(args, cfg).await?,
        Command::L2ToL1LogProof(args) => get_l2_to_l1_proof::run(args, cfg).await?,
        Command::MainContract(args) => get_main_contract::run(args, cfg).await?,
        Command::BridgehubContract(args) => get_bridgehub_contract::run(args, cfg).await?,
        Command::TestnetPaymaster(args) => testnet_paymaster::run(args, cfg).await?,
        Command::L1ChainID => get_l1_chain_id::run(cfg).await?,
        Command::L1BaseTokenAddress(args) => get_l1_base_token_address::run(args, cfg).await?,
        Command::AllAccountBalances(args) => get_all_account_balances::run(args, cfg).await?,
        Command::L1BatchNumber => get_l1_batch_number::run(cfg).await?,
        Command::BlockDetails(args) => get_block_details::run(args, cfg).await?,
        Command::TransactionDetails(args) => transaction_details::run(args, cfg).await?,
        Command::L1GasPrice => get_l1_gas_price::run(cfg).await?,
        Command::FeeParams => get_fee_params::run(cfg).await?,
        Command::ProtocolVersion(args) => get_protocol_version::run(args, cfg).await?,
        Command::Balance(args) => balance::run(args, cfg).await?,
        Command::FinalizeDepositTx(args) => finalize_deposit_transaction::run(args, cfg).await?,
    };

    Ok(())
}
