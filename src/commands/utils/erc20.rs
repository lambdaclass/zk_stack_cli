use std::sync::Arc;
use zksync_ethers_rs::{
    core::k256::ecdsa::SigningKey,
    providers::{Http, Provider},
    signers::Wallet,
    types::{Address, U256},
    zk_wallet::ZKWallet,
};

use ethers_contract::abigen;

abigen!(
    MINT_IERC20,
    r"[function mint(address _to, uint256 _amount) nonpayable returns (bool)]"
);

pub(crate) async fn erc20_mint(
    erc20_token_address: Address,
    wallet: ZKWallet<Provider<Http>, Wallet<SigningKey>>,
    amount: U256,
    from_l1: bool,
) -> eyre::Result<bool> {
    let provider = if from_l1 {
        wallet.l1_provider()
    } else {
        wallet.l2_provider()
    };

    let erc20_contract = MINT_IERC20::new(erc20_token_address, Arc::new(provider.clone()));
    let result = erc20_contract.mint(wallet.l1_address(), amount).await?;
    Ok(result)
}
