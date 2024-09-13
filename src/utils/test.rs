use super::balance::{display_balance, get_erc20_balance, get_erc20_balance_decimals_symbol};
use crate::utils::{contract::*, wallet::new_zkwallet};
use colored::Colorize;
use eyre::ContextCompat;
use spinoff::{spinners, Color, Spinner};
use std::{ops::Div, sync::Arc};
use tokio::task::JoinSet;
use zksync_ethers_rs::{
    contracts::erc20::MINT_IERC20,
    core::k256::ecdsa::SigningKey,
    core::rand::{thread_rng, Rng},
    core::utils::{format_ether, parse_ether},
    providers::{Http, Middleware, Provider, ProviderError},
    signers::{LocalWallet, Signer, Wallet},
    types::{
        transaction::eip2718::TypedTransaction, Address, Eip1559TransactionRequest, L2TxOverrides,
        TransactionReceipt, U256,
    },
    types::{Bytes, H256},
    zk_wallet::ZKWallet,
    ZKMiddleware,
};

pub async fn send_transactions(
    from_wallet: &Arc<ZKWallet<Provider<Http>, LocalWallet>>,
    to_wallets: &Vec<Arc<ZKWallet<Provider<Http>, LocalWallet>>>,
    parsed_amount: U256,
) -> eyre::Result<Vec<H256>> {
    println!(
        "{} Transfer from {} wallet to {} wallet.",
        "[L2->L2]".bold().bright_cyan().on_black(),
        "rich".bold().red().on_black(),
        "each".bold().blue().on_black()
    );

    let mut l2_txs_receipts: Vec<H256> = Vec::new();
    let mut set = JoinSet::new();

    let mut nonce = from_wallet
        .l2_provider()
        .get_transaction_count(from_wallet.l2_address(), None)
        .await?;

    for w in to_wallets {
        let from_wallet_clone = Arc::clone(from_wallet);
        let to = w.l2_address();
        set.spawn(async move {
            from_wallet_clone
                .transfer_base_token(parsed_amount, to, Some(L2TxOverrides::new().nonce(nonce)))
                .await
        });
        nonce = nonce.saturating_add(U256::one());
    }

    println!(
        "{}",
        "Waiting for all transactions to finish".yellow().on_black()
    );

    while let Some(res) = set.join_next().await {
        match res {
            Ok(Ok(tx_hash)) => {
                l2_txs_receipts.push(tx_hash);
            }
            Ok(Err(_)) | Err(_) => {
                println!("Error in tx");
            }
        };
    }
    Ok(l2_txs_receipts)
}

pub async fn send_transactions_back(
    from_wallets: &Vec<Arc<ZKWallet<Provider<Http>, LocalWallet>>>,
    to_wallet: &Arc<ZKWallet<Provider<Http>, LocalWallet>>,
) -> eyre::Result<Vec<H256>> {
    println!(
        "{} Transfer from {} wallet to {} wallet.",
        "[L2->L2]".bold().bright_cyan().on_black(),
        "each".bold().blue().on_black(),
        "rich".bold().red().on_black()
    );

    let mut l2_txs_receipts: Vec<H256> = Vec::new();
    let mut set = JoinSet::new();

    for w in from_wallets {
        let to_wallet_clone = Arc::clone(to_wallet);
        let from_wallet_clone = Arc::clone(w);
        set.spawn(async move {
            future_transfer_base_token_back(&from_wallet_clone, &to_wallet_clone).await
        });
    }

    while let Some(res) = set.join_next().await {
        match res {
            Ok(Ok(tx_hash)) => {
                l2_txs_receipts.push(tx_hash);
            }
            Ok(Err(_)) | Err(_) => {
                println!("Error in tx");
            }
        };
    }

    println!(
        "{}",
        "Waiting for all transactions to finish".yellow().on_black()
    );

    Ok(l2_txs_receipts)
}

pub async fn future_transfer_base_token_back(
    from_wallet: &ZKWallet<Provider<Http>, LocalWallet>,
    to_wallet: &ZKWallet<Provider<Http>, LocalWallet>,
) -> eyre::Result<H256> {
    let balance = from_wallet
        .l2_provider()
        .get_balance(from_wallet.l2_address(), None)
        .await?;
    let transfer_tx = TypedTransaction::Eip1559(
        Eip1559TransactionRequest::new()
            .from(from_wallet.l2_address())
            .to(to_wallet.l2_address())
            .value(balance),
    );
    let gas_estimate = from_wallet
        .l2_provider()
        .estimate_gas(&transfer_tx, None)
        .await?
        .div(10_u32)
        .saturating_mul(U256::from(11_u32)); // 10% of headroom
    let gas_price = from_wallet.l2_provider().get_gas_price().await?;
    let gas = gas_estimate.saturating_mul(gas_price);
    let transfer_hash = from_wallet
        .transfer_base_token(
            balance.saturating_sub(gas),
            to_wallet.l1_address(),
            // The nonce is not changed since all the transfers are from different wallets
            None,
        )
        .await?;
    Ok(transfer_hash)
}

pub async fn deposit_base_token(
    from_wallet: &Arc<ZKWallet<Provider<Http>, LocalWallet>>,
    parsed_amount_to_deposit: U256,
    verbose: bool,
) -> eyre::Result<()> {
    if verbose {
        println!(
            "{} Deposit from {} wallet to {} wallet.",
            "[L1->L2]".bold().bright_cyan().on_black(),
            "rich".bold().red().on_black(),
            "rich".bold().red().on_black()
        );
    }
    from_wallet
        .deposit_base_token(parsed_amount_to_deposit)
        .await?;
    Ok(())
}

pub async fn get_n_random_wallets(
    number_of_wallets: u16,
    l1_provider: &Provider<Http>,
    l2_provider: &Provider<Http>,
) -> eyre::Result<Vec<Arc<ZKWallet<Provider<Http>, LocalWallet>>>> {
    let mut wallets = Vec::new();
    for i in 1..=number_of_wallets {
        let local_wallet = LocalWallet::new(&mut thread_rng());
        let pk_bytes = local_wallet.signer().to_bytes();
        let pk = hex::encode(pk_bytes);
        println!(
            "Wallet [{i:0>3}] addr: {:?} || pk: 0x{pk}",
            local_wallet.address(),
        );
        let w = new_zkwallet(local_wallet, l1_provider, l2_provider).await?;
        wallets.push(Arc::new(w));
    }
    Ok(wallets)
}

pub async fn display_balances(
    wallets: &[Arc<ZKWallet<Provider<Http>, LocalWallet>>],
) -> eyre::Result<()> {
    for (i, w) in wallets.iter().enumerate() {
        println!("{}", "=".repeat(64));
        println!("Wallet [{i:0>3}] addr: {:?}", w.l2_address());
        display_balance(None, w, false, true).await?;
        println!("{}", "=".repeat(64));
    }
    Ok(())
}

pub async fn erc20_l1_mint(
    erc20_token_address: Address,
    wallet: &ZKWallet<Provider<Http>, Wallet<SigningKey>>,
    amount: U256,
) -> eyre::Result<TransactionReceipt> {
    let erc20_contract = MINT_IERC20::new(erc20_token_address, wallet.l1_signer());
    let tx_receipt = erc20_contract
        .mint(wallet.l1_address(), amount)
        .send()
        .await?
        .await?
        .context("No transaction receipt for erc20 mint")?;

    Ok(tx_receipt)
}

pub async fn check_balance_and_deposit_or_mint(
    wallet: Arc<ZKWallet<Provider<Http>, LocalWallet>>,
    base_token_address: Address,
    amount: U256,
) -> eyre::Result<()> {
    let mut spinner = Spinner::new(spinners::Dots, "Checking L2 Balance", Color::Blue);

    let l2_balance = wallet
        .l2_provider()
        .get_balance(wallet.l1_address(), None)
        .await?;

    if l2_balance.le(&amount) {
        let (l1_balance, _, token_symbol) = get_erc20_balance_decimals_symbol(
            base_token_address,
            wallet.l1_address(),
            wallet.l1_provider(),
        )
        .await?;

        spinner.update(spinners::Dots, "Checking L1 Balance", Color::Blue);

        // Here we are assuming that the base token has 18 decimals
        if parse_ether(&l1_balance)?.le(&amount) {
            let mint_amount = amount;

            let balance = get_erc20_balance(
                base_token_address,
                wallet.l1_address(),
                wallet.l1_provider(),
            )
            .await?;

            let msg = format!(
                "[L1] Balance isn't enough: {balance} {token_symbol} || Minting {} {token_symbol}",
                format_ether(mint_amount)
            );
            spinner.update(spinners::Dots, msg, Color::Blue);

            let future_receipt = erc20_l1_mint(base_token_address, &wallet, mint_amount);

            let receipt = future_receipt.await?;

            let msg = format!("Success, Mint TxHash: {:?}", receipt.transaction_hash);
            spinner.update(spinners::Dots, msg, Color::Blue);
        }
        spinner.update(spinners::Dots, "Depositing", Color::Blue);
        // Begin Deposit from rich wallet to rich wallet
        deposit_base_token(&wallet, amount, false).await?;
        // End Deposit from rich wallet to rich wallet
        spinner.success("Success, Deposit");
    } else {
        spinner.success("Enough L2 balance");
    }
    Ok(())
}

// Contract Being Used
// SPDX-License-Identifier: UNLICENSED
//pragma solidity ^0.8.13;
//
//contract StorageFibonacci {
//    uint256 public storedData;
//
//    function set(uint256 x) public {
//        storedData = fibonacci(x);
//    }
//
//    function get() public view returns (uint256) {
//        return storedData;
//    }
//
//    // Calculate Fibonacci
//    // Cannot be accessed from outside the contract
//    function fibonacci(uint256 x) internal pure returns (uint256) {
//        if (x == 0) return 0;
//        if (x == 1) return 1;
//
//        uint256 a = 0;
//        uint256 b = 1;
//        uint256 c;
//
//        for (uint256 i = 2; i <= x; i++) {
//            c = a + b;
//            a = b;
//            b = c;
//        }
//
//        return b;
//    }
//}
pub(crate) async fn send_contract_transactions_for_test(
    from_wallet: &Arc<ZKWallet<Provider<Http>, LocalWallet>>,
    contract_address: Address,
    tpr: u64,
) -> eyre::Result<Vec<H256>> {
    println!(
        "{} Contract Interaction",
        "[L2->L2]".bold().bright_cyan().on_black(),
    );

    let function_signature = "set(uint256)";

    let mut l2_txs_receipts: Vec<H256> = Vec::new();
    let mut set = JoinSet::new();

    let mut nonce = from_wallet
        .l2_provider()
        .get_transaction_count(from_wallet.l2_address(), None)
        .await?;

    let chain_id = from_wallet.l2_provider().get_chainid().await?;

    let mut rng = thread_rng();
    let mut random_values = vec![];
    for _ in 0..tpr {
        random_values.push(rng.gen_range(50_i32..150_i32));
    }

    for r in random_values {
        let from_wallet_clone = Arc::clone(from_wallet);
        set.spawn(async move {
            println!("r: {r}");
            let selector = get_fn_selector(function_signature);
            let types = parse_signature(function_signature).map_err(|e| {
                ProviderError::CustomError(format!("Error parsing signature: {e:?}"))
            })?;

            let tx_data = encode_call(Some(selector), None, Some(vec![r.to_string()]), types)
                .map_err(|e| ProviderError::CustomError(format!("Error encoding call: {e:?}")))?;

            let tx_data_bytes = Bytes::from(tx_data);

            let mut raw_tx = Eip1559TransactionRequest::new()
                .to(contract_address)
                .data(tx_data_bytes)
                .value(U256::zero())
                .from(from_wallet_clone.l2_address())
                .nonce(nonce)
                .chain_id(chain_id.as_u64());

            let tx: TypedTransaction = raw_tx.clone().into();
            let fees = from_wallet_clone
                .l2_provider()
                .estimate_fee(&tx)
                .await
                .map_err(|e| ProviderError::CustomError(format!("Error estimating fee: {e:?}")))?;

            raw_tx = raw_tx
                .max_fee_per_gas(fees.max_fee_per_gas)
                .max_priority_fee_per_gas(fees.max_priority_fee_per_gas)
                .gas(fees.gas_limit.div(10_i32).saturating_mul(11_i32.into())); // headroom 10% extra

            let tx: TypedTransaction = raw_tx.into();

            from_wallet_clone
                .l2_signer()
                .send_transaction(tx, None)
                .await
                .map_err(|e| {
                    ProviderError::CustomError(format!("Error sending transaction: {e:?}"))
                })?
                .await
        });
        nonce = nonce.saturating_add(U256::one());
    }

    while let Some(res) = set.join_next().await {
        match res {
            Ok(Ok(Some(tx_receipt))) => {
                l2_txs_receipts.push(tx_receipt.transaction_hash);
            }
            Ok(Err(_)) | Err(_) | Ok(Ok(None)) => {
                println!("Error in tx");
            }
        };
    }
    Ok(l2_txs_receipts)
}
