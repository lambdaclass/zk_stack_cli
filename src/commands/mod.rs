pub(crate) mod account_balance;
pub(crate) mod call;
pub(crate) mod compile;
pub(crate) mod deploy;
pub(crate) mod encode;
pub(crate) mod get_bridge_contracts;
pub(crate) mod get_bytecode_by_hash;
pub(crate) mod get_contract;
pub(crate) mod get_transaction;
pub(crate) mod pay;
pub(crate) mod selector;

// It is set so that the transaction is replay-protected (EIP-155)
// https://era.zksync.io/docs/api/hardhat/testing.html#connect-wallet-to-local-nodes
#[allow(dead_code)]
const L1_CHAIN_ID: u64 = 9;
const L2_CHAIN_ID: u64 = 270;
