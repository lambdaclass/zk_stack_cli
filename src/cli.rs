use crate::commands::{
    account_balance, call, compile, deploy, encode, get_bridge_contracts, get_bytecode_by_hash,
    get_contract, get_transaction, pay, selector,
};
use clap::{command, Args, Parser, Subcommand};

pub const VERSION_STRING: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser)]
#[command(name="zksync-era-cli", author, version=VERSION_STRING, about, long_about = None)]
struct ZKSyncCLI {
    #[command(subcommand)]
    command: ZKSyncCommand,
    #[clap(flatten)]
    config: ZKSyncConfig,
}

#[derive(Args)]
pub struct ZKSyncConfig {
    #[clap(long, default_value = "localhost")]
    pub host: String,
    #[clap(short, long, default_value = "3050")]
    pub port: u16,
}

#[derive(Subcommand)]
enum ZKSyncCommand {
    Deploy(deploy::Args),
    Call(call::Args),
    GetContract(get_contract::Args),
    GetTransaction(get_transaction::Args),
    Balance(account_balance::Args),
    Pay(pay::Args),
    Compile(compile::Args),
    Encode(encode::Args),
    Selector(selector::Args),
    GetBridgeContracts,
    GetBytecodeByHash(get_bytecode_by_hash::Args),
}

pub async fn start() -> eyre::Result<()> {
    let ZKSyncCLI { command, config } = ZKSyncCLI::parse();
    match command {
        ZKSyncCommand::Deploy(args) => deploy::run(args, config).await?,
        ZKSyncCommand::Call(args) => call::run(args, config).await?,
        ZKSyncCommand::GetContract(args) => get_contract::run(args, config).await?,
        ZKSyncCommand::GetTransaction(args) => get_transaction::run(args, config).await?,
        ZKSyncCommand::Balance(args) => account_balance::run(args, config).await?,
        ZKSyncCommand::Pay(args) => pay::run(args, config).await?,
        ZKSyncCommand::Compile(args) => {
            let _ = compile::run(args)?;
        }
        ZKSyncCommand::Encode(args) => encode::run(args).await?,
        ZKSyncCommand::Selector(args) => selector::run(args).await?,
        ZKSyncCommand::GetBridgeContracts => get_bridge_contracts::run(config).await?,
        ZKSyncCommand::GetBytecodeByHash(args) => {
            get_bytecode_by_hash::run(args, config).await?
        }
    };

    Ok(())
}
