use std::str::FromStr;

use crate::{
    config::ZKSyncConfig,
    utils::balance::{display_l1_balance, display_l2_balance},
};
use clap::Subcommand;
use eyre::ContextCompat;
use zksync_ethers_rs::{
    abi::Hash,
    middleware::SignerMiddleware,
    providers::{Middleware, Provider},
    signers::{LocalWallet, Signer},
    types::{Address, U256},
    zk_wallet::ZKWallet,
    ZKMiddleware,
};

#[derive(Subcommand)]
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
    #[clap(about = "Deposit funds into the wallet.")]
    Deposit {
        #[clap(long = "amount", value_parser=U256::from_dec_str)]
        amount: U256,
        #[clap(long = "token")]
        token_address: Option<Address>,
        #[clap(long = "from")]
        from: Option<LocalWallet>,
        #[clap(long = "to")]
        to: Option<Address>,
        #[clap(long, required = false)]
        explorer_url: bool,
    },
    #[clap(about = "Finalize a pending withdrawal.")]
    FinalizeWithdraw {
        #[clap(long = "hash")]
        l2_withdraw_tx_hash: Hash,
        #[clap(long = "to")]
        to: Option<Address>,
    },
    #[clap(about = "Transfer funds to another wallet.")]
    Transfer {
        #[clap(long = "amount", value_parser = U256::from_dec_str)]
        amount: U256,
        #[clap(long = "token")]
        token_address: Option<Address>,
        #[clap(long = "from")]
        from: Option<LocalWallet>,
        #[clap(long = "to")]
        to: Address,
        #[clap(long, required = false)]
        explorer_url: bool,
    },
    #[clap(about = "Withdraw funds from the wallet. TODO.")]
    Withdraw,
    #[clap(about = "Get the wallet address.")]
    Address,
    #[clap(about = "Get the wallet private key.")]
    PrivateKey,
}

impl Command {
    pub async fn run(self, cfg: ZKSyncConfig) -> eyre::Result<()> {
        let wallet_config = cfg.wallet.clone().context("Wallet config missing")?;
        match self {
            Command::Balance {
                token_address,
                l2,
                l1,
            } => {
                let l1_provider = Provider::try_from(
                    cfg.network
                        .l1_rpc_url
                        .context("L1 RPC URL missing in config")?,
                )?;
                let l2_provider = Provider::try_from(cfg.network.l2_rpc_url)?;
                let base_token_address = l2_provider.get_base_token_l1_address().await?;

                if l2 || !l1 {
                    display_l2_balance(
                        wallet_config.address,
                        token_address,
                        &l1_provider,
                        &l2_provider,
                        base_token_address,
                        l1,
                    )
                    .await?;
                };
                if l1 {
                    display_l1_balance(wallet_config.address, token_address, &l1_provider).await?;
                };
            }
            Command::Deposit {
                amount,
                token_address,
                from,
                to,
                explorer_url,
            } => {
                let l1_provider = Provider::try_from(
                    cfg.network
                        .l1_rpc_url
                        .context("L1 RPC URL missing in config")?,
                )?;
                let l1_chain_id = l1_provider.get_chainid().await?.as_u64();
                let wallet = from
                    .unwrap_or(wallet_config.private_key.parse()?)
                    .with_chain_id(l1_chain_id);
                let l1_signer = SignerMiddleware::new(l1_provider, wallet.clone());

                let l2_provider = Provider::try_from(cfg.network.l2_rpc_url)?;
                let l1_chain_id = l2_provider.get_chainid().await?.as_u64();
                let wallet = wallet.with_chain_id(l1_chain_id);
                let l2_signer = SignerMiddleware::new(l2_provider, wallet);

                let zk_wallet = ZKWallet::new(l1_signer, l2_signer);

                let deposit_hash = match (to, token_address) {
                    (None, None) => zk_wallet.deposit_base_token(amount).await?,
                    (None, Some(token)) => zk_wallet.deposit_erc20(amount, token).await?,
                    (Some(to), None) => zk_wallet.deposit_base_token_to(amount, to).await?,
                    (Some(to), Some(token)) => {
                        zk_wallet.deposit_erc20_to(amount, token, to).await?
                    }
                };

                if explorer_url {
                    let url = cfg
                        .network
                        .l1_explorer_url
                        .context("L1 Explorer URL missing in config")?;
                    println!("Deposit: {url}/tx/{deposit_hash:?}");
                } else {
                    println!("Deposit hash: {deposit_hash:?}");
                }
            }
            Command::FinalizeWithdraw {
                l2_withdraw_tx_hash,
                to: _to,
            } => {
                let l2_provider = Provider::try_from(cfg.network.l2_rpc_url)?;
                let l1_provider =
                    Provider::try_from(cfg.network.l1_rpc_url.context("L1 RPC URL is needed")?)?;
                let wallet = LocalWallet::from_str(
                    &cfg.wallet.context("Wallet config missing")?.private_key,
                )?;
                let signer = SignerMiddleware::new(l1_provider, wallet);
                zksync_ethers_rs::finalize_withdrawal(
                    signer.into(),
                    l2_withdraw_tx_hash,
                    &l2_provider,
                )
                .await;
            }
            Command::Transfer {
                amount,
                token_address,
                from,
                to,
                explorer_url,
            } => {
                let l1_provider = Provider::try_from(
                    cfg.network
                        .l1_rpc_url
                        .context("L1 RPC URL missing in config")?,
                )?;
                let l1_chain_id = l1_provider.get_chainid().await?.as_u64();
                let wallet = from
                    .unwrap_or(wallet_config.private_key.parse()?)
                    .with_chain_id(l1_chain_id);
                let l1_signer = SignerMiddleware::new(l1_provider, wallet.clone());

                let l2_provider = Provider::try_from(cfg.network.l2_rpc_url)?;
                let l2_chain_id = l2_provider.get_chainid().await?.as_u64();
                let wallet = wallet.with_chain_id(l2_chain_id);
                let l2_signer = SignerMiddleware::new(l2_provider, wallet);

                let zk_wallet = ZKWallet::new(l1_signer, l2_signer);

                let transfer_hash = if let Some(token_address) = token_address {
                    zk_wallet
                        .transfer_erc20(amount, token_address, to, None)
                        .await?
                } else {
                    zk_wallet.transfer_base_token(amount, to, None).await?
                };

                if explorer_url {
                    let url = cfg
                        .network
                        .l2_explorer_url
                        .context("L2 Explorer URL missing in config")?;
                    println!("Transfer: {url}/tx/{transfer_hash:?}");
                } else {
                    println!("Transfer hash: {transfer_hash:?}");
                }
            }
            Command::Withdraw => todo!("Withdraw"),
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
