use crate::config::ZKSyncConfig;
use crate::utils::balance::get_erc20_decimals_symbol;
use crate::utils::gas_tracker::{self, GasTracker};
use crate::utils::{
    balance::{display_balance, get_erc20_balance_decimals_symbol},
    test::*,
    wallet::*,
};
use clap::Subcommand;
use colored::*;
use core::time;
use eyre::ContextCompat;
use spinoff::{spinners, Color, Spinner};
use std::{
    ops::{Add, Div},
    sync::Arc,
    thread::sleep,
};
use zksync_ethers_rs::{
    core::utils::{format_ether, parse_ether},
    providers::Middleware,
    types::U256,
    wait_for_finalize_withdrawal, ZKMiddleware,
};

#[derive(Subcommand)]
pub(crate) enum Command {
    #[clap(about = "LoadTest the zkStack Chain.", visible_alias = "lt")]
    LoadTest {
        #[clap(long = "wallets", short = 'w', required = true)]
        number_of_wallets: u16,
        #[clap(
            long = "amount",
            short = 'a',
            required = true,
            help = "Amount of BaseToken to deposit, 20% more will be deposited.\nThat extra 20% will remain in the main wallet,\nthe rest will be redistributed to random wallets"
        )]
        amount: f32,
        #[clap(
            long = "reruns",
            short = 'r',
            help = "If set to 0 it will run indefinitely, If not set defaults to 1 run."
        )]
        reruns_wanted: Option<u8>,
        #[arg(
            long = "withdraw",
            short = 'w',
            default_value_t = false,
            help = "If set, the funds will be withdrawn after each run."
        )]
        withdraw: bool,
        #[arg(
            long = "sleep",
            short = 's',
            default_value_t = 1,
            help = "Sleep interval between each rerun"
        )]
        sleep_secs: u64,
    },
    #[clap(
        about = "Gas Measurements for the zkStack Chain.",
        visible_alias = "gs"
    )]
    GasScenario {
        #[clap(long = "tps", required = true)]
        tps: u64,
        #[clap(
            long = "amount",
            short = 'a',
            required = true,
            help = "Amount of BaseToken to deposit, 20% more will be deposited.\nThat extra 20% will remain in the main wallet,\nthe rest will be redistributed to random wallets"
        )]
        amount: f32,
        #[arg(
            long = "rounds",
            short = 'r',
            default_value_t = 1,
            help = "Amount of times to run the program in a loop, it defaults to 1. Max is 255"
        )]
        rounds: u8,
    },
}

impl Command {
    pub async fn run(self, cfg: ZKSyncConfig) -> eyre::Result<()> {
        let (zk_wallet, l1_provider, l2_provider) = get_wallet_l1_l2_providers(cfg)?;
        let base_token_address = l2_provider.get_base_token_l1_address().await?;

        match self {
            Command::LoadTest {
                number_of_wallets,
                amount,
                reruns_wanted,
                withdraw,
                sleep_secs,
            } => {
                let wallets =
                    get_n_random_wallets(number_of_wallets, &l1_provider, &l2_provider).await?;
                // ideally it should be the amount transferred, the gas + fees have to be deducted automatically
                let parsed_amount_to_deposit = parse_ether(amount)?
                    .div(10_u32)
                    .saturating_mul(U256::from(12_u32)); // 20% of headroom
                let float_wallets: f32 = number_of_wallets.into();
                let amount_of_bt_to_transfer_for_each: f32 = amount / float_wallets;
                let amount_of_bt_to_withdraw: f32 = amount;
                // Here we are assuming that the base token has 18 decimals
                let parsed_amount_of_bt_to_transfer_for_each =
                    parse_ether(amount_of_bt_to_transfer_for_each)?;

                // Begin Display L1 Balance and BaseToken Addr
                println!("{}", "#".repeat(64));
                println!(
                    "{}: {base_token_address:?}",
                    "Base Token Address".bold().green().on_black()
                );
                display_balance(None, &zk_wallet, true, false).await?;
                display_balance(Some(base_token_address), &zk_wallet, true, false).await?;

                println!("{}", "#".repeat(64));
                // End Display L1 Balance and BaseToken Addr

                let mut reruns = 0;
                let mut current_reruns: u32 = 1;
                let reruns_wanted = reruns_wanted.unwrap_or(1);
                let reruns_to_complete = if reruns_wanted == 0 { 1 } else { reruns_wanted };

                println!(
                    "Number of reruns {}",
                    if reruns_wanted == 0 {
                        "∞".to_owned().red()
                    } else {
                        reruns_wanted.to_string().red()
                    }
                );
                let zk_wallet_addr = zk_wallet.l2_address();
                let arc_zk_wallet = Arc::new(zk_wallet);
                while reruns < reruns_to_complete {
                    println!(
                        "{} N: {}",
                        "Run".red().on_black(),
                        (current_reruns).to_string().yellow().on_black()
                    );

                    let mut spinner =
                        Spinner::new(spinners::Dots, "Checking L2 Balance", Color::Blue);

                    let l2_balance = l2_provider.get_balance(zk_wallet_addr, None).await?;

                    if l2_balance.le(&parsed_amount_to_deposit) {
                        spinner.update(spinners::Dots, "Checking L1 Balance", Color::Blue);

                        let (l1_balance, _, token_symbol) = get_erc20_balance_decimals_symbol(
                            base_token_address,
                            zk_wallet_addr,
                            &l1_provider,
                        )
                        .await?;

                        // Here we are assuming that the base token has 18 decimals
                        if parse_ether(l1_balance)?.le(&parsed_amount_to_deposit) {
                            let mint_amount = parsed_amount_to_deposit;

                            let msg = format!(
                                "Not enough tokens... Minting {} {token_symbol}",
                                format_ether(mint_amount)
                            );
                            spinner.update(spinners::Dots, msg, Color::Blue);

                            let future_receipt =
                                erc20_l1_mint(base_token_address, &arc_zk_wallet, mint_amount);

                            let receipt = future_receipt.await?;

                            display_balance(Some(base_token_address), &arc_zk_wallet, true, false)
                                .await?;
                            println!("Transaction Hash: {:?}", receipt.transaction_hash);
                        }
                        spinner.update(spinners::Dots, "Depositing", Color::Blue);
                        // Begin Deposit from rich wallet to rich wallet
                        deposit_base_token(&arc_zk_wallet, parsed_amount_to_deposit).await?;
                        // End Deposit from rich wallet to rich wallet
                        spinner.success("Success, Deposit");
                    } else {
                        spinner.success("Enough L2 balance");
                    }

                    // Begin Transfer from rich wallet to each wallet

                    display_balances(&wallets).await?;

                    println!(
                        "{} Transfer from {} wallet to {} wallet.",
                        "[L2->L2]".bold().bright_cyan().on_black(),
                        "rich".bold().red().on_black(),
                        "each".bold().blue().on_black()
                    );
                    println!(
                        "{}",
                        "Waiting for all transactions to finish".yellow().on_black()
                    );

                    let _tx_hashes = send_transactions(
                        &arc_zk_wallet,
                        &wallets,
                        parsed_amount_of_bt_to_transfer_for_each,
                    )
                    .await?;

                    display_balances(&wallets).await?;

                    // End Transfer from rich wallet to each wallet
                    println!("{}", "#".repeat(64));
                    // Begin Transfer from each wallet to rich wallet

                    display_balance(None, &arc_zk_wallet, false, true).await?;

                    println!(
                        "{} Transfer from {} wallet to {} wallet.",
                        "[L1->L2]".bold().bright_cyan().on_black(),
                        "each".bold().blue().on_black(),
                        "rich".bold().red().on_black()
                    );

                    let _tx_hashes = send_transactions_back(&wallets, &arc_zk_wallet).await?;

                    display_balance(None, &arc_zk_wallet, false, true).await?;

                    // End Transfer from each wallet to rich wallet
                    println!("{}", "#".repeat(64));

                    if withdraw {
                        // Begin Withdrawal
                        println!(
                            "{} Withdraw basetoken from {} wallet.",
                            "[L2->L1]".bold().bright_cyan().on_black(),
                            "rich".bold().red().on_black(),
                        );

                        display_balance(Some(base_token_address), &arc_zk_wallet, true, true)
                            .await?;
                        let withdraw_hash = arc_zk_wallet
                            .withdraw_base_token(parse_ether(amount_of_bt_to_withdraw.to_string())?)
                            .await?;
                        println!("Withdraw hash: {withdraw_hash:?}");
                        let base_token_address =
                            Some(l2_provider.get_base_token_l1_address().await?);
                        println!("finalize withdrawal");
                        wait_for_finalize_withdrawal(withdraw_hash, &l2_provider).await;
                        arc_zk_wallet.finalize_withdraw(withdraw_hash).await?;
                        display_balance(base_token_address, &arc_zk_wallet, true, true).await?;
                        println!("{}", "#".repeat(64));
                        // End Withdrawal
                    }

                    if reruns_wanted != 0 {
                        reruns += 1;
                    }
                    current_reruns += 1;

                    let mut spinner = Spinner::new(
                        spinners::Dots,
                        format!("Waiting for {sleep_secs} second(s)"),
                        Color::Blue,
                    );
                    sleep(time::Duration::from_secs(sleep_secs));
                    spinner.success(&format!("Rerun {current_reruns} finished"));
                }
                Ok(())
            }
            Command::GasScenario {
                tps,
                amount,
                rounds,
            } => {
                // TPS, at the moment the calculation is performed with the following conditions
                // - Don't take deposits into account
                // - 1 transaction from the rich wallet to each random wallet
                // - 1 transaction from each random wallet to the rich wallet
                // - sleep 1 second (this has to be revised, maybe not necesary) and start again. This step is made to simulate the number of transaction per second
                //  - (disclaimer) this tps is not the same tps of the chain, here we are sending transactions, the chain is processing transactions
                //  - (disclaimer) moreover, the "downtime" of the deposit transaction is not taken into account. This reduces the actual tps.
                //  - the only variable is the amount of randowm wallets
                //  - taking into account we will have 2*number_of_wallets transactions
                //  - the number_of_wallets is calculated as follows: 2*number_of_wallets [txs] = tps [tx/s] * 1 [s]
                //  - so number_of_wallets = (tps+1)/2. The +1 is to have a round number. For example, 3/2 = 1 but 4/2 = 2
                let number_of_wallets = ((tps + 1) / 2).try_into()?;
                // There is a flag called rounds, which tells the amount of times to run the prorgram,
                // the following assumptions are made
                //  - If 1 round lasts 1 second to send all the transactions per second specified by the tps flag
                //    the runtime would be rounds * 1[s] = rounds [seconds]
                //  - If we follow the sleep condition to simulate the transactions per second, this may vary.

                let mut gas_tracker = GasTracker::new();

                let mut txs_per_run;
                let mut fees_sma_per_run;
                let mut gas_sma_per_run;
                let mut gas_sma_price_per_run;

                let wallets =
                    get_n_random_wallets(number_of_wallets, &l1_provider, &l2_provider).await?;
                // ideally it should be the amount transferred, the gas + fees have to be deducted automatically
                let parsed_amount_to_deposit = parse_ether(amount)?
                    .div(10_u32)
                    .saturating_mul(U256::from(12_u32)); // 20% of headroom
                let float_wallets: f32 = number_of_wallets.into();
                let amount_of_bt_to_transfer_for_each: f32 = amount / float_wallets;

                // Here we are assuming that the base token has 18 decimals
                let parsed_amount_of_bt_to_transfer_for_each =
                    parse_ether(amount_of_bt_to_transfer_for_each)?;

                let zk_wallet_addr = zk_wallet.l2_address();
                let arc_zk_wallet = Arc::new(zk_wallet);

                let mut reruns: u8 = 0;
                let mut current_reruns: u8 = 1;
                let reruns_wanted: u8 = rounds;
                let reruns_to_complete: u8 = if reruns_wanted == 0 { 1 } else { reruns_wanted };

                while reruns < reruns_to_complete {
                    fees_sma_per_run = U256::zero();
                    gas_sma_per_run = U256::zero();
                    gas_sma_price_per_run = U256::zero();

                    let mut spinner =
                        Spinner::new(spinners::Dots, "Checking L2 Balance", Color::Blue);

                    let l2_balance = l2_provider.get_balance(zk_wallet_addr, None).await?;

                    println!(
                        "{} N: {}",
                        "Run".red().on_black(),
                        (current_reruns).to_string().yellow().on_black()
                    );

                    if l2_balance.le(&parsed_amount_to_deposit) {
                        let (l1_balance, _, token_symbol) = get_erc20_balance_decimals_symbol(
                            base_token_address,
                            zk_wallet_addr,
                            &l1_provider,
                        )
                        .await?;

                        spinner.update(spinners::Dots, "Checking L1 Balance", Color::Blue);

                        // Here we are assuming that the base token has 18 decimals
                        if parse_ether(&l1_balance)?.le(&parsed_amount_to_deposit) {
                            let mint_amount = parsed_amount_to_deposit;

                            let msg = format!(
                                "Not enough tokens... Minting {} {token_symbol}",
                                format_ether(mint_amount)
                            );
                            spinner.update(spinners::Dots, msg, Color::Blue);

                            let future_receipt =
                                erc20_l1_mint(base_token_address, &arc_zk_wallet, mint_amount);

                            let receipt = future_receipt.await?;

                            display_balance(Some(base_token_address), &arc_zk_wallet, true, false)
                                .await?;
                            println!("Transaction Hash: {:?}", receipt.transaction_hash);
                        }
                        spinner.update(spinners::Dots, "Depositing", Color::Blue);
                        // Begin Deposit from rich wallet to rich wallet
                        deposit_base_token(&arc_zk_wallet, parsed_amount_to_deposit).await?;
                        // End Deposit from rich wallet to rich wallet
                        spinner.success("Success, Deposit");
                    } else {
                        spinner.success("Enough L2 balance");
                    }

                    // Begin Transfer from rich wallet to each wallet

                    println!(
                        "{} Transfer from {} wallet to {} wallet.",
                        "[L2->L2]".bold().bright_cyan().on_black(),
                        "rich".bold().red().on_black(),
                        "each".bold().blue().on_black()
                    );

                    println!(
                        "{}",
                        "Waiting for all transactions to finish".yellow().on_black()
                    );

                    let tx_hashes_forwards = send_transactions(
                        &arc_zk_wallet,
                        &wallets,
                        parsed_amount_of_bt_to_transfer_for_each,
                    )
                    .await?;

                    display_balances(&wallets).await?;

                    // End Transfer from rich wallet to each wallet
                    println!("{}", "#".repeat(64));
                    // Begin Transfer from each wallet to rich wallet

                    display_balance(None, &arc_zk_wallet, false, true).await?;

                    println!(
                        "{} Transfer from {} wallet to {} wallet.",
                        "[L1->L2]".bold().bright_cyan().on_black(),
                        "each".bold().blue().on_black(),
                        "rich".bold().red().on_black()
                    );

                    let tx_hashes_backwards =
                        send_transactions_back(&wallets, &arc_zk_wallet).await?;

                    // End Transfer from each wallet to rich wallet
                    println!("{}", "#".repeat(64));

                    let mut tx_hashes = tx_hashes_forwards.clone();
                    tx_hashes.extend(&tx_hashes_backwards);
                    txs_per_run = tx_hashes.len().try_into()?;

                    for h in tx_hashes {
                        let receipt = l2_provider
                            .get_transaction_receipt(h)
                            .await?
                            .context("Error unwrapping tx_receipt")?;

                        let gas_used = receipt.gas_used.context("Error unwrapping gas_used")?;
                        let receipt_gas_price = receipt
                            .effective_gas_price
                            .context("Error unwrapping gas price")?;

                        let details = l2_provider
                            .get_transaction_details(h)
                            .await?
                            .context("Error unwrapping tx_details")?;

                        // Implementing simple moving average (SMA)
                        if gas_sma_per_run.is_zero() {
                            gas_sma_per_run = gas_used;
                        }
                        gas_sma_per_run = gas_sma_per_run.add(gas_used) / 2_u32;
                        if fees_sma_per_run.is_zero() {
                            fees_sma_per_run = details.fee;
                        }
                        fees_sma_per_run = fees_sma_per_run.add(details.fee) / 2_u32;
                        if gas_sma_price_per_run.is_zero() {
                            gas_sma_price_per_run = receipt_gas_price;
                        }
                        gas_sma_price_per_run = (receipt_gas_price + gas_sma_price_per_run) / 2_u32;
                    }

                    gas_tracker.add_run(
                        gas_sma_per_run,
                        fees_sma_per_run,
                        gas_sma_price_per_run,
                        txs_per_run,
                    );

                    if reruns_wanted != 0 {
                        reruns += 1;
                    }
                    current_reruns += 1;
                }
                println!("{gas_tracker}");
                Ok(())
            }
        }
    }
}
