use crate::config::ZKSyncConfig;
use clap::Subcommand;

#[derive(Subcommand)]
pub(crate) enum Command {}

impl Command {
    pub async fn run(self, _cfg: ZKSyncConfig) -> eyre::Result<()> {
        match self {};
    }
}
