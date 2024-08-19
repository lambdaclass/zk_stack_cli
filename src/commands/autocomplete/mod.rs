use std::fs::{File, OpenOptions};
use std::io::{self, Write};

use clap::Args as ClapArgs;
use clap::{CommandFactory, Subcommand};
use clap_complete::{aot::Shell, generate};

use crate::cli::ZKSyncCLI;

#[derive(ClapArgs, PartialEq)]
pub(crate) struct Args {
    #[clap(short = 's', long = "shell", help = "Default: $SHELL")]
    pub shell: Option<Shell>,
}

#[derive(Subcommand, PartialEq)]
pub(crate) enum Command {
    #[clap(about = "Generate autocomplete shell script.")]
    Generate(Args),
    #[clap(about = "Generate and install autocomplete shell script.")]
    Install(Args),
}

fn get_shellrc_path(shell: Shell) -> Option<String> {
    match shell {
        Shell::Bash => Some(".bashrc".to_string()),
        Shell::Zsh => Some(".zshrc".to_string()),
        Shell::Fish => Some(".config/fish/config.fish".to_string()),
        Shell::Elvish => Some(".elvish/rc.elv".to_string()),
        Shell::PowerShell => {
            Some(".config/powershell/Microsoft.PowerShell_profile.ps1".to_string())
        }
        _ => None,
    }
}

fn generate_bash_script(shell: Option<Shell>) {
    let shell = shell.unwrap_or(Shell::from_env().unwrap());
    generate(shell, &mut ZKSyncCLI::command(), "zks", &mut io::stdout());
}

fn install_bash_script(shell: Option<Shell>) {
    let shell = shell.unwrap_or(Shell::from_env().unwrap());
    let file_path = dirs::home_dir().unwrap().join(".zks-completion");
    let mut file = File::create(&file_path).unwrap();
    generate(shell, &mut ZKSyncCLI::command(), "zks", &mut file);
    file.flush().unwrap();

    let shellrc_path = dirs::home_dir()
        .unwrap()
        .join(get_shellrc_path(shell).unwrap());
    let mut file = OpenOptions::new().append(true).open(shellrc_path).unwrap();
    if shell == Shell::Elvish {
        file.write_all(b"\n-source $HOME/.zks-completion\n")
            .unwrap();
    } else if shell == Shell::PowerShell {
        file.write_all(format!("\n. {}\n", file_path.as_path().display()).as_bytes())
            .unwrap();
    } else {
        file.write_all(b"\n. $HOME/.zks-completion\n").unwrap();
    }
    file.flush().unwrap();

    println!("Autocomplete script installed. To apply changes, restart your shell.");
}

pub(crate) fn start(cmd: Command) {
    match cmd {
        Command::Generate(args) => generate_bash_script(args.shell),
        Command::Install(args) => install_bash_script(args.shell),
    };
}
