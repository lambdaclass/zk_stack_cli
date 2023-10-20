use super::{output::ZKSArtifact, project::ZKSProject};
use std::{ffi::OsString, path::PathBuf, str::FromStr};
use zksync_web3_rs::{
    prelude::{abi::Abi, info::ContractInfo, Project, ProjectPathsConfig},
    solc::{utils::source_files, ConfigurableContractArtifact},
    types::Bytes,
};

#[derive(Clone)]
pub enum Compiler {
    ZKSolc,
    Solc,
}

impl From<OsString> for Compiler {
    fn from(compiler: OsString) -> Self {
        match compiler.to_str().unwrap() {
            "zksolc" => Compiler::ZKSolc,
            "solc" => Compiler::Solc,
            _ => panic!("Invalid compiler"),
        }
    }
}

#[derive(Debug)]
pub enum Artifact {
    ZKSArtifact(ZKSArtifact),
    SolcArtifact(ConfigurableContractArtifact),
}

impl Artifact {
    pub fn abi(&self) -> Abi {
        match self {
            Artifact::ZKSArtifact(artifact) => artifact.abi.clone().unwrap(),
            Artifact::SolcArtifact(artifact) => artifact.abi.clone().unwrap().abi,
        }
    }

    pub fn bin(&self) -> Bytes {
        match self {
            Artifact::ZKSArtifact(artifact) => artifact.bin.clone().unwrap(),
            Artifact::SolcArtifact(artifact) => artifact
                .bytecode
                .clone()
                .unwrap()
                .object
                .as_bytes()
                .unwrap()
                .clone(),
        }
    }
}

pub fn compile(contract_path: &str, contract_name: &str, compiler: Compiler) -> Artifact {
    match compiler {
        Compiler::ZKSolc => compile_with_zksolc(contract_path, contract_name),
        Compiler::Solc => compile_with_solc(contract_path, contract_name),
    }
}

fn compile_with_zksolc(contract_path: &str, contract_name: &str) -> Artifact {
    let root = PathBuf::from(contract_path);
    let zk_project = ZKSProject::from(
        Project::builder()
            .paths(ProjectPathsConfig::builder().build_with_root(root))
            .set_auto_detect(true)
            .build()
            .unwrap(),
    );
    let compilation_output = zk_project.compile().unwrap();
    let artifact = compilation_output
        .find_contract(ContractInfo::from_str(&format!("{contract_path}:{contract_name}")).unwrap())
        .unwrap()
        .clone();
    Artifact::ZKSArtifact(artifact)
}

fn compile_with_solc(contract_path: &str, contract_name: &str) -> Artifact {
    let root = PathBuf::from(contract_path);
    let project = Project::builder()
        .paths(ProjectPathsConfig::builder().build_with_root(root))
        .set_auto_detect(true)
        .build()
        .unwrap();
    let compilation_output = project.compile().unwrap();
    let artifact = compilation_output
        .find_contract(ContractInfo::from_str(&format!("{contract_path}:{contract_name}")).unwrap())
        .unwrap()
        .clone();
    Artifact::SolcArtifact(artifact)
}

pub fn build(contract_path: &str, compiler: Compiler) {
    match compiler {
        Compiler::ZKSolc => build_with_zksolc(contract_path),
        Compiler::Solc => build_with_solc(contract_path),
    }
}

fn build_with_zksolc(contract_path: &str) {
    let root = PathBuf::from(contract_path);
    let zk_project = ZKSProject::from(
        Project::builder()
            .paths(ProjectPathsConfig::builder().build_with_root(root))
            .set_auto_detect(true)
            .build()
            .unwrap(),
    );
    zk_project.build().unwrap();
}

fn build_with_solc(contract_path: &str) {
    let root = PathBuf::from(contract_path);
    let project = Project::builder()
        .paths(ProjectPathsConfig::builder().build_with_root(root))
        .set_auto_detect(true)
        .build()
        .unwrap();

    let solc_path = PathBuf::from("src/compiler/bin/solc");

    let command = &mut std::process::Command::new(solc_path);
    command
        .arg("@openzeppelin/=node_modules/@openzeppelin/")
        .arg("--combined-json")
        .arg("abi,bin")
        .arg("--output-dir")
        .arg("contracts/build/")
        .arg("--")
        .args(source_files(project.root()));

    let command_output = command.output().unwrap();

    log::info!(
        "stdout: {}",
        String::from_utf8_lossy(&command_output.stdout)
            .into_owned()
            .trim()
            .to_owned()
    );
    log::info!(
        "stderr: {}",
        String::from_utf8_lossy(&command_output.stderr)
            .into_owned()
            .trim()
            .to_owned()
    );
}
