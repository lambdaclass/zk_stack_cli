use crate::config::ZKSyncConfig;
use clap::Subcommand;

pub(crate) mod prover;
pub(crate) mod server;

#[derive(Subcommand)]
pub(crate) enum Command {
    #[clap(subcommand, visible_alias = "s")]
    Server(server::Command),
    #[clap(subcommand, visible_alias = "p")]
    Prover(prover::Command),
}

impl Command {
    pub async fn run(self, cfg: ZKSyncConfig) -> eyre::Result<()> {
        match self {
            Command::Server(cmd) => cmd.run(cfg).await?,
            Command::Prover(cmd) => cmd.run(cfg).await?,
        };

        Ok(())
    }
}
