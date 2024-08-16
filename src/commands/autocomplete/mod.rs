use std::io;

use clap::{CommandFactory, Subcommand};
use clap::Args as ClapArgs;
use clap_complete::{aot::Shell, generate};

use crate::cli::ZKSyncCLI;

#[derive(ClapArgs, PartialEq)]
pub(crate) struct Args {
    #[clap(long = "shell", default_value = "bash")]
    pub shell: Shell,
}

#[derive(Subcommand, PartialEq)]
pub(crate) enum Command {
    #[clap(about = "Generate autocomplete shell script.")]
    Generate(Args),
}

fn generate_bash_script(shell: Shell) {
    generate(shell, &mut ZKSyncCLI::command(), "zks", &mut io::stdout());
}

pub(crate) fn start(cmd: Command) {
    match cmd {
        Command::Generate(args) => generate_bash_script(args.shell)
    };
}
