use eyre::ContextCompat;

/*
mkdir -p 'src/compiler/bin' \
    && wget 'https://github.com/matter-labs/zksolc-bin/raw/main/macosx-arm64/zksolc-macosx-arm64-v1.3.14' -O 'src/compiler/bin/zksolc' \
    && chmod +x 'src/compiler/bin/zksolc' \
    && brew install solidity \
    && cp /opt/homebrew/bin/solc src/compiler/bin/solc
*/
pub(crate) fn run() -> eyre::Result<()> {
    let target = dirs::config_dir()
        .context("config dir not found")?
        .join("eth-compilers");
    std::fs::create_dir_all(&target)?;
    let target = target.to_str().context("empty path")?;
    std::process::Command::new("mkdir").args(["-p", target, "\\"])
        .args(["&&", "wget", "https://github.com/matter-labs/zksolc-bin/raw/main/macosx-arm64/zksolc-macosx-arm64-v1.3.14", "-O", &format!("{target}/zksolc"), "\\"])
        .args(["&&", "chmod", "+x", &format!("{target}/zksolc")])
        .args(["&&", "brew", "install", "solidity"])
        .args(["&&", "cp",  "/opt/homebrew/bin/solc", &format!("{target}/solc"), "\\"])
        .spawn()?;
    Ok(())
}
