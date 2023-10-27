use clap::Args as ClapArgs;

pub mod compiler;
pub mod errors;
pub mod output;
pub mod project;

#[derive(ClapArgs)]
pub(crate) struct Args {
    #[clap(short, long, name = "COMPILER")]
    pub compiler: compiler::Compiler,
    #[clap(short, long, name = "PROJECT_ROOT_PATH")]
    pub project_root: String,
    #[clap(short, long, name = "CONTRACT_PATH")]
    pub contract_path: String,
    #[clap(short, long, name = "CONTRACT_NAME")]
    pub contract_name: String,
}

pub(crate) async fn run(args: Args) -> eyre::Result<()> {
    let output = compiler::compile(
        &args.project_root,
        &args.contract_path,
        &args.contract_name,
        args.compiler,
    )?;
    log::info!("{output:?}");
    Ok(())
}
