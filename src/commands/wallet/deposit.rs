use clap::Args as ClapArgs;
use zksync_ethers_rs::{
    signers::LocalWallet,
    types::{Address, U256},
};

use crate::config::ZKSyncConfig;

#[derive(ClapArgs)]
pub(crate) struct Args {
    #[clap(long = "amount")]
    pub amount: U256,
    #[clap(long = "token")]
    pub token_address: Option<Address>,
    #[clap(long = "from")]
    pub from: Option<LocalWallet>,
    #[clap(long = "to")]
    pub to: Option<Address>,
}

pub(crate) async fn run(args: Args, _cfg: ZKSyncConfig) -> eyre::Result<()> {
    match (args.from, args.to) {
        (None, None) => todo!("Self Deposit"),
        (None, Some(_)) => todo!("Deposit to another account"),
        (Some(_), None) => todo!("Deposit from another account"),
        (Some(_), Some(_)) => todo!("Deposit from another account to another account"),
    }
}
