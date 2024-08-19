use std::fs::{File, OpenOptions};
use std::io::{self, Write};

use clap::{Args as ClapArgs, ValueEnum};
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

fn get_shellrc_path(shell: Shell) -> eyre::Result<String> {
    match shell {
        Shell::Bash => Ok(".bashrc".to_owned()),
        Shell::Zsh => Ok(".zshrc".to_owned()),
        Shell::Fish => Ok(".config/fish/config.fish".to_owned()),
        Shell::Elvish => Ok(".elvish/rc.elv".to_owned()),
        Shell::PowerShell => Ok(".config/powershell/Microsoft.PowerShell_profile.ps1".to_owned()),
        _ => Err(eyre::eyre!(
            "Your shell is not supported. Supported shells are: {:?}",
            Shell::value_variants()
        )),
    }
}

fn get_shell(arg: Option<Shell>) -> eyre::Result<Shell> {
    if let Some(shell) = arg {
        Ok(shell)
    } else if let Some(env_shell) = Shell::from_env() {
        Ok(env_shell)
    } else {
        Err(eyre::eyre!(
            "Your shell is not supported. Supported shells are: {:?}",
            Shell::value_variants()
        ))
    }
}

fn generate_bash_script(shell_arg: Option<Shell>) -> eyre::Result<()> {
    let shell = get_shell(shell_arg)?;
    generate(shell, &mut ZKSyncCLI::command(), "zks", &mut io::stdout());
    Ok(())
}

fn install_bash_script(shell_arg: Option<Shell>) -> eyre::Result<()> {
    let shell = get_shell(shell_arg)?;

    let file_path = dirs::home_dir()
        .ok_or(eyre::eyre!("Cannot find home directory."))?
        .join(".zks-completion");
    let mut file = File::create(&file_path)?;
    generate(shell, &mut ZKSyncCLI::command(), "zks", &mut file);
    file.flush()?;

    let shellrc_path = dirs::home_dir()
        .ok_or(eyre::eyre!("Cannot find home directory."))?
        .join(get_shellrc_path(shell)?);
    let mut file = OpenOptions::new().append(true).open(shellrc_path)?;
    if shell == Shell::Elvish {
        file.write_all(b"\n-source $HOME/.zks-completion\n")?;
    } else if shell == Shell::PowerShell {
        file.write_all(format!("\n. {}\n", file_path.as_path().display()).as_bytes())?;
    } else {
        file.write_all(b"\n. $HOME/.zks-completion\n")?;
    }
    file.flush()?;

    println!("Autocomplete script installed. To apply changes, restart your shell.");
    Ok(())
}

pub(crate) fn start(cmd: Command) -> eyre::Result<()> {
    match cmd {
        Command::Generate(args) => generate_bash_script(args.shell),
        Command::Install(args) => install_bash_script(args.shell),
    }
}
