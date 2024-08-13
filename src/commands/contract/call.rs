use clap::Args as ClapArgs;

use crate::config::ZKSyncConfig;

#[derive(ClapArgs)]
pub(crate) struct Args {
    #[clap(long = "contract")]
    pub contract_address: String,
    #[clap(long = "function")]
    pub function_name: String,
    #[clap(long = "args")]
    pub args: Vec<String>,
}

pub(crate) async fn run(args: Args, cfg: ZKSyncConfig) -> eyre::Result<()> {
    todo!("Call")
}
