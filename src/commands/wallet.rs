use crate::config::ZKSyncConfig;
use crate::utils::balance::display_balance;
use crate::utils::wallet::get_wallet_l1_l2_providers;
use clap::Subcommand;
use eyre::ContextCompat;
use spinoff::{spinners, Color, Spinner};
use zksync_ethers_rs::{
    abi::Hash, core::utils::parse_ether, types::Address, wait_for_finalize_withdrawal, ZKMiddleware,
};

#[derive(Subcommand, PartialEq)]
pub(crate) enum Command {
    #[clap(about = "Get the balance of the wallet.")]
    Balance {
        #[clap(long = "token")]
        token_address: Option<Address>,
        #[clap(long = "l2", required = false)]
        l2: bool,
        #[clap(long = "l1", required = false)]
        l1: bool,
    },
    #[clap(about = "Deposit funds into some wallet.")]
    Deposit {
        #[clap(long = "amount")]
        amount: String,
        #[clap(
            long = "token",
            help = "Specify the token address, the base token is used as default."
        )]
        token_address: Option<Address>,
        #[clap(
            long = "to",
            help = "Specify the wallet in which you want to deposit yout funds."
        )]
        to: Option<Address>,
    },
    #[clap(about = "Finalize a pending withdrawal.")]
    FinalizeWithdraw {
        #[clap(long = "hash")]
        l2_withdrawal_tx_hash: Hash,
    },
    #[clap(about = "Transfer funds to another wallet.")]
    Transfer {
        #[clap(long = "amount")]
        amount: String,
        #[clap(long = "token")]
        token_address: Option<Address>,
        #[clap(long = "to")]
        to: Address,
        #[clap(
            long = "l1",
            required = false,
            help = "If set it will do an L1 transfer, defaults to an L2 transfer"
        )]
        l1: bool,
    },
    #[clap(about = "Withdraw funds from the wallet. TODO.")]
    Withdraw {
        #[clap(long = "amount")]
        amount: String,
        #[clap(
            long = "token",
            help = "Specify the token address, the base token is used as default."
        )]
        token_address: Option<Address>,
    },
    #[clap(about = "Get the wallet address.")]
    Address,
    #[clap(about = "Get the wallet private key.")]
    PrivateKey,
}

// TODO Handle ETH
impl Command {
    pub async fn run(self, cfg: ZKSyncConfig) -> eyre::Result<()> {
        let wallet_config = cfg
            .clone()
            .wallet
            .clone()
            .context("Wallet config missing")?;

        let l1_explorer_url = cfg
            .clone()
            .network
            .l1_explorer_url
            .unwrap_or("https://sepolia.etherscan.io".to_owned());

        let l2_explorer_url = cfg
            .clone()
            .network
            .l2_explorer_url
            .unwrap_or("http://localhost:3010".to_owned());

        let (zk_wallet, _l1_provider, l2_provider) = get_wallet_l1_l2_providers(cfg)?;
        let base_token_address = l2_provider.get_base_token_l1_address().await?;

        match self {
            Command::Balance {
                token_address,
                l2,
                l1,
            } => display_balance(token_address, &zk_wallet, l1, l2).await?,
            Command::Deposit {
                amount,
                token_address,
                to,
            } => {
                // Assuming all tokens have 18 decimals
                // TODO revise this
                let parsed_amount = parse_ether(&amount)?;
                let deposit_hash = match (to, token_address) {
                    (None, None) => zk_wallet.deposit_base_token(parsed_amount).await?,
                    (None, Some(token)) => zk_wallet.deposit_erc20(parsed_amount, token).await?,
                    (Some(to), None) => zk_wallet.deposit_base_token_to(parsed_amount, to).await?,
                    (Some(to), Some(token)) => {
                        zk_wallet.deposit_erc20_to(parsed_amount, token, to).await?
                    }
                };

                println!("Deposit: {l1_explorer_url}/tx/{deposit_hash:?}");
            }
            Command::FinalizeWithdraw {
                l2_withdrawal_tx_hash,
            } => {
                let msg = "Waiting for Withdrawal Finalization";
                let mut spinner = Spinner::new(spinners::Arrow3, msg, Color::Cyan);

                let wait_withdraw =
                    wait_for_finalize_withdrawal(l2_withdrawal_tx_hash, &l2_provider);
                wait_withdraw.await;
                spinner.success("Withdrawal finalized!");
                let withdraw_hash = zk_wallet.finalize_withdraw(l2_withdrawal_tx_hash).await?;

                println!("Withdraw: {l1_explorer_url}/tx/{withdraw_hash:?}");
            }
            Command::Transfer {
                amount,
                token_address,
                to,
                l1,
            } => {
                // Assuming all tokens have 18 decimals
                // TODO revise this
                let parsed_amount = parse_ether(&amount)?;

                if l1 {
                    todo!("L1 transfers not supported by ZKWallet");
                } else {
                    let transfer_hash = if let Some(token_address) = token_address {
                        zk_wallet
                            .transfer_erc20(parsed_amount, token_address, to, None)
                            .await?
                    } else {
                        zk_wallet
                            .transfer_base_token(parsed_amount, to, None)
                            .await?
                    };
                    println!("Withdraw: {l2_explorer_url}/tx/{transfer_hash:?}");
                }
            }
            Command::Withdraw {
                amount,
                token_address,
            } => {
                // Assuming all tokens have 18 decimals
                // TODO revise this
                let parsed_amount = parse_ether(&amount)?;

                let msg = "Waiting for Withdrawal Finalization";
                let mut spinner: Spinner = Spinner::new(spinners::Arrow3, msg, Color::Cyan);
                // TODO revise how to withdraw ETH
                let l2_withdrawal_tx_hash = if let Some(token) = token_address {
                    if token == base_token_address {
                        zk_wallet.withdraw_base_token(parsed_amount).await?
                    } else {
                        zk_wallet.withdraw_erc20(parsed_amount, token).await?
                    }
                } else {
                    zk_wallet.withdraw_base_token(parsed_amount).await?
                };
                let wait_withdraw =
                    wait_for_finalize_withdrawal(l2_withdrawal_tx_hash, &l2_provider);
                wait_withdraw.await;
                zk_wallet.finalize_withdraw(l2_withdrawal_tx_hash).await?;
                spinner.success("Withdrawal finalized!");
            }
            Command::Address => {
                println!("Wallet address: {:?}", wallet_config.address);
            }
            Command::PrivateKey => {
                println!("Wallet private key: {:?}", wallet_config.private_key);
            }
        };

        Ok(())
    }
}
