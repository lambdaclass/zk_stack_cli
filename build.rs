use std::{fs, path::PathBuf};

// wget 'https://github.com/matter-labs/zksolc-bin/raw/main/macosx-arm64/zksolc-macosx-arm64-v1.3.14' -O 'src/compiler/bin/zksolc'
fn download_zksolc(target: PathBuf) {
    std::process::Command::new("wget")
        .arg("https://github.com/matter-labs/zksolc-bin/raw/main/macosx-arm64/zksolc-macosx-arm64-v1.3.14")
        .arg("-O")
        .arg(target.clone())
        .spawn()
        .unwrap();
    // target.push("zksolc");
    // std::process::Command::new("chmod")
    //     .arg("+x")
    //     .arg(target)
    //     .spawn()
    //     .unwrap();
}

fn download_solc(mut target: PathBuf) {
    std::process::Command::new("brew")
        .arg("install")
        .arg("solidity")
        .spawn()
        .unwrap();

    target.push("solc");

    std::process::Command::new("cp")
        .arg("/opt/homebrew/bin/solc")
        .arg(target)
        .spawn()
        .unwrap();
}

fn main() {
    let target = dirs::config_dir().unwrap().join("eth-compilers");
    fs::create_dir_all(&target).unwrap();
    download_zksolc(target.clone());
    download_solc(target);
}
