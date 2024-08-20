use crate::config::ZKSyncConfig;
use clap::Subcommand;

#[derive(Subcommand, PartialEq)]
pub(crate) enum Command {
    #[clap(about = "Call view functions on a contract.")]
    Call {
        contract_address: String,
        function_name: String,
        args: Vec<String>,
    },
    #[clap(about = "Deploy a contract.")]
    Deploy {
        bytecode: String,
        constructor_args: Vec<String>,
    },
    #[clap(about = "Call non-view functions on a contract.")]
    Send {
        contract_address: String,
        function_name: String,
        args: Vec<String>,
    },
}

impl Command {
    pub fn run(self, _cfg: ZKSyncConfig) -> eyre::Result<()> {
        match self {
            Command::Call {
                contract_address: _,
                function_name: _,
                args: _,
            } => todo!("Call"),
            Command::Deploy {
                bytecode: _,
                constructor_args: _,
            } => todo!("Deploy"),
            Command::Send {
                contract_address: _,
                function_name: _,
                args: _,
            } => todo!(),
        };
    }
}
