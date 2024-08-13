use clap::Args as ClapArgs;

use crate::config::ZKSyncConfig;

#[derive(ClapArgs)]
pub(crate) struct Args {
    #[clap(long = "bytecode")]
    pub bytecode: String,
    #[clap(long = "constructor")]
    pub constructor_args: Vec<String>,
}

pub(crate) async fn run(args: Args, cfg: ZKSyncConfig) -> eyre::Result<()> {
    todo!("Deploy")
}
