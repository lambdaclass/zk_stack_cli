use super::db::{
    for_batch_queries::*,
    types::{
        BasicWitnessGeneratorJobInfo, LeafWitnessGeneratorJobInfo, NodeWitnessGeneratorJobInfo,
        ProofCompressionJobInfo, ProverJobFriInfo, RecursionTipWitnessGeneratorJobInfo,
        SchedulerWitnessGeneratorJobInfo, StageFlags,
    },
};
use circuit_definitions::zkevm_circuits::scheduler::aux::BaseLayerCircuitType;
use colored::Colorize;
use sqlx::{pool::PoolConnection, Postgres};
use std::collections::BTreeMap;
use strum::{Display, EnumString};
use zksync_ethers_rs::types::zksync::{
    basic_fri_types::AggregationRound,
    prover_dal::{
        ExtendedJobCountStatistics, ProofCompressionJobStatus, ProverJobStatus, Stallable,
        WitnessJobStatus,
    },
    L1BatchNumber,
};

// From: zksync-era:
// https://github.com/matter-labs/zksync-era/blob/main/prover/crates/bin/prover_cli/src/commands/status/utils.rs#L34
#[derive(Default, Debug, EnumString, Clone, Display)]
pub enum Status {
    /// A custom status that can be set manually.
    /// Mostly used when a task has singular status.
    Custom(String),
    /// A task is considered queued when all of its jobs is queued.
    #[strum(to_string = "Queued 📥")]
    Queued,
    /// A task is considered in progress when at least one of its jobs differs in its status.
    #[strum(to_string = "In Progress ⌛️")]
    InProgress,
    /// A task is considered successful when all of its jobs were processed successfully.
    #[strum(to_string = "Successful ✅")]
    Successful,
    /// A task is considered waiting for proofs when all of its jobs are waiting for proofs.
    #[strum(to_string = "Waiting for Proof ⏱️")]
    WaitingForProofs,
    /// A task is considered stuck when at least one of its jobs is stuck.
    #[strum(to_string = "Stuck ⛔️")]
    Stuck,
    /// A task has no jobs.
    #[default]
    #[strum(to_string = "Jobs not found 🚫")]
    JobsNotFound,
}

impl From<ProverJobStatus> for Status {
    fn from(status: ProverJobStatus) -> Self {
        match status {
            ProverJobStatus::Queued => Status::Queued,
            ProverJobStatus::InProgress(_) => Status::InProgress,
            ProverJobStatus::Successful(_) => Status::Successful,
            ProverJobStatus::Failed(_) => Status::Custom("Failed".to_owned()),
            ProverJobStatus::Skipped => Status::Custom("Skipped ⏩".to_owned()),
            ProverJobStatus::Ignored => Status::Custom("Ignored".to_owned()),
            ProverJobStatus::InGPUProof => Status::Custom("In GPU Proof".to_owned()),
        }
    }
}

impl From<WitnessJobStatus> for Status {
    fn from(status: WitnessJobStatus) -> Self {
        match status {
            WitnessJobStatus::Queued => Status::Queued,
            WitnessJobStatus::InProgress => Status::InProgress,
            WitnessJobStatus::Successful(_) => Status::Successful,
            WitnessJobStatus::Failed(_) => Status::InProgress,
            WitnessJobStatus::WaitingForArtifacts => {
                Status::Custom("Waiting for Artifacts ⏱️".to_owned())
            }
            WitnessJobStatus::Skipped => Status::Custom("Skipped ⏩".to_owned()),
            WitnessJobStatus::WaitingForProofs => Status::WaitingForProofs,
        }
    }
}

impl From<Vec<WitnessJobStatus>> for Status {
    fn from(status_vector: Vec<WitnessJobStatus>) -> Self {
        if status_vector.is_empty() {
            Status::JobsNotFound
        } else if status_vector
            .iter()
            .all(|job| matches!(job, WitnessJobStatus::WaitingForProofs))
        {
            Status::WaitingForProofs
        } else if status_vector.iter().all(|job| {
            matches!(job, WitnessJobStatus::Queued)
                || matches!(job, WitnessJobStatus::WaitingForProofs)
        }) {
            Status::Queued
        } else if status_vector
            .iter()
            .all(|job| matches!(job, WitnessJobStatus::Successful(_)))
        {
            Status::Successful
        } else {
            Status::InProgress
        }
    }
}

impl From<Vec<LeafWitnessGeneratorJobInfo>> for Status {
    fn from(leaf_info_vector: Vec<LeafWitnessGeneratorJobInfo>) -> Self {
        leaf_info_vector
            .iter()
            .map(|s| s._status.clone())
            .collect::<Vec<WitnessJobStatus>>()
            .into()
    }
}

impl From<Vec<NodeWitnessGeneratorJobInfo>> for Status {
    fn from(node_info_vector: Vec<NodeWitnessGeneratorJobInfo>) -> Self {
        node_info_vector
            .iter()
            .map(|s| s._status.clone())
            .collect::<Vec<WitnessJobStatus>>()
            .into()
    }
}

impl From<Vec<RecursionTipWitnessGeneratorJobInfo>> for Status {
    fn from(scheduler_info_vector: Vec<RecursionTipWitnessGeneratorJobInfo>) -> Self {
        scheduler_info_vector
            .iter()
            .map(|s| s._status.clone())
            .collect::<Vec<WitnessJobStatus>>()
            .into()
    }
}

impl From<Vec<SchedulerWitnessGeneratorJobInfo>> for Status {
    fn from(scheduler_info_vector: Vec<SchedulerWitnessGeneratorJobInfo>) -> Self {
        scheduler_info_vector
            .iter()
            .map(|s| s._status.clone())
            .collect::<Vec<WitnessJobStatus>>()
            .into()
    }
}

impl From<ProofCompressionJobStatus> for Status {
    fn from(status: ProofCompressionJobStatus) -> Self {
        match status {
            ProofCompressionJobStatus::Queued => Status::Queued,
            ProofCompressionJobStatus::InProgress => Status::InProgress,
            ProofCompressionJobStatus::Successful => Status::Successful,
            ProofCompressionJobStatus::Failed => Status::InProgress,
            ProofCompressionJobStatus::SentToServer => {
                Status::Custom("Sent to server 📤".to_owned())
            }
            ProofCompressionJobStatus::Skipped => Status::Custom("Skipped ⏩".to_owned()),
        }
    }
}

// From: zksync-era:
// https://github.com/matter-labs/zksync-era/blob/main/prover/crates/bin/prover_cli/src/commands/status/utils.rs#L171
#[allow(clippy::large_enum_variant, reason = "strum")]
#[derive(EnumString, Clone, Display, Debug)]
pub enum StageInfo {
    #[strum(to_string = "Basic Witness Generator")]
    BasicWitnessGenerator {
        witness_generator_job_info: Option<BasicWitnessGeneratorJobInfo>,
        prover_jobs_info: Vec<ProverJobFriInfo>,
    },
    #[strum(to_string = "Leaf Witness Generator")]
    LeafWitnessGenerator {
        witness_generator_jobs_info: Vec<LeafWitnessGeneratorJobInfo>,
        prover_jobs_info: Vec<ProverJobFriInfo>,
    },
    #[strum(to_string = "Node Witness Generator")]
    NodeWitnessGenerator {
        witness_generator_jobs_info: Vec<NodeWitnessGeneratorJobInfo>,
        prover_jobs_info: Vec<ProverJobFriInfo>,
    },
    #[strum(to_string = "Recursion Tip")]
    RecursionTipWitnessGenerator(Option<RecursionTipWitnessGeneratorJobInfo>),
    #[strum(to_string = "Scheduler")]
    SchedulerWitnessGenerator(Option<SchedulerWitnessGeneratorJobInfo>),
    #[strum(to_string = "Compressor")]
    Compressor(Option<ProofCompressionJobInfo>),
}

impl StageInfo {
    pub fn aggregation_round(&self) -> Option<AggregationRound> {
        match self {
            StageInfo::BasicWitnessGenerator { .. } => Some(AggregationRound::BasicCircuits),
            StageInfo::LeafWitnessGenerator { .. } => Some(AggregationRound::LeafAggregation),
            StageInfo::NodeWitnessGenerator { .. } => Some(AggregationRound::NodeAggregation),
            StageInfo::RecursionTipWitnessGenerator { .. } => Some(AggregationRound::RecursionTip),
            StageInfo::SchedulerWitnessGenerator { .. } => Some(AggregationRound::Scheduler),
            StageInfo::Compressor(_) => None,
        }
    }

    pub fn prover_jobs_status(&self, max_attempts: u32) -> Option<Status> {
        match self.clone() {
            StageInfo::BasicWitnessGenerator {
                prover_jobs_info, ..
            }
            | StageInfo::LeafWitnessGenerator {
                prover_jobs_info, ..
            }
            | StageInfo::NodeWitnessGenerator {
                prover_jobs_info, ..
            } => Some(get_prover_jobs_status_from_vec(
                &prover_jobs_info,
                max_attempts,
            )),
            StageInfo::RecursionTipWitnessGenerator(_)
            | StageInfo::SchedulerWitnessGenerator(_)
            | StageInfo::Compressor(_) => None,
        }
    }

    pub fn witness_generator_jobs_status(&self, max_attempts: u32) -> Status {
        match self.clone() {
            StageInfo::BasicWitnessGenerator {
                witness_generator_job_info,
                ..
            } => witness_generator_job_info
                .map(|witness_generator_job_info| {
                    get_witness_generator_job_status(&witness_generator_job_info, max_attempts)
                })
                .unwrap_or_default(),
            StageInfo::LeafWitnessGenerator {
                witness_generator_jobs_info,
                ..
            } => {
                get_witness_generator_job_status_from_vec(witness_generator_jobs_info, max_attempts)
            }
            StageInfo::NodeWitnessGenerator {
                witness_generator_jobs_info,
                ..
            } => {
                get_witness_generator_job_status_from_vec(witness_generator_jobs_info, max_attempts)
            }
            StageInfo::RecursionTipWitnessGenerator(witness_generator_job_info) => {
                witness_generator_job_info
                    .map(|witness_generator_job_info| {
                        get_witness_generator_job_status(&witness_generator_job_info, max_attempts)
                    })
                    .unwrap_or_default()
            }
            StageInfo::SchedulerWitnessGenerator(witness_generator_job_info) => {
                witness_generator_job_info
                    .map(|witness_generator_job_info| {
                        get_witness_generator_job_status(&witness_generator_job_info, max_attempts)
                    })
                    .unwrap_or_default()
            }
            StageInfo::Compressor(status) => status
                .map(|job| Status::from(job._status))
                .unwrap_or_default(),
        }
    }
}

pub fn get_witness_generator_job_status(data: &impl Stallable, max_attempts: u32) -> Status {
    let status = data.get_status();
    if matches!(
        status,
        WitnessJobStatus::Failed(_) | WitnessJobStatus::InProgress,
    ) && data.get_attempts() >= max_attempts
    {
        return Status::Stuck;
    }
    Status::from(status)
}

pub fn get_witness_generator_job_status_from_vec(
    prover_jobs: Vec<impl Stallable>,
    max_attempts: u32,
) -> Status {
    if prover_jobs.is_empty() {
        Status::JobsNotFound
    } else if prover_jobs
        .iter()
        .all(|job| matches!(job.get_status(), WitnessJobStatus::WaitingForProofs))
    {
        Status::WaitingForProofs
    } else if prover_jobs.iter().any(|job| {
        matches!(
            job.get_status(),
            WitnessJobStatus::Failed(_) | WitnessJobStatus::InProgress,
        ) && job.get_attempts() >= max_attempts
    }) {
        Status::Stuck
    } else if prover_jobs.iter().all(|job| {
        matches!(job.get_status(), WitnessJobStatus::Queued)
            || matches!(job.get_status(), WitnessJobStatus::WaitingForProofs)
    }) {
        Status::Queued
    } else if prover_jobs
        .iter()
        .all(|job| matches!(job.get_status(), WitnessJobStatus::Successful(_)))
    {
        Status::Successful
    } else {
        Status::InProgress
    }
}

pub fn get_prover_job_status(prover_jobs: ProverJobFriInfo, max_attempts: u32) -> Status {
    if matches!(
        prover_jobs._status,
        ProverJobStatus::Failed(_) | ProverJobStatus::InProgress(_),
    ) && prover_jobs._attempts >= max_attempts
    {
        return Status::Stuck;
    }
    Status::from(prover_jobs._status)
}

pub fn get_prover_jobs_status_from_vec(
    prover_jobs: &[ProverJobFriInfo],
    max_attempts: u32,
) -> Status {
    if prover_jobs.is_empty() {
        Status::JobsNotFound
    } else if prover_jobs.iter().any(|job| {
        matches!(
            job._status,
            ProverJobStatus::Failed(_) | ProverJobStatus::InProgress(_),
        ) && job._attempts >= max_attempts
    }) {
        Status::Stuck
    } else if prover_jobs
        .iter()
        .all(|job| matches!(job._status, ProverJobStatus::InGPUProof))
    {
        Status::Custom("In GPU Proof ⚡️".to_owned())
    } else if prover_jobs
        .iter()
        .all(|job| matches!(job._status, ProverJobStatus::Queued))
    {
        Status::Queued
    } else if prover_jobs
        .iter()
        .all(|job| matches!(job._status, ProverJobStatus::Successful(_)))
    {
        Status::Successful
    } else {
        Status::InProgress
    }
}

#[derive(Debug)]
/// Represents the proving data of a l1_batch_number.
pub struct BatchData {
    /// The number of the l1_batch_number.
    pub batch_number: L1BatchNumber,
    /// The basic witness generator data.
    pub basic_witness_generator: StageInfo,
    /// The leaf witness generator data.
    pub leaf_witness_generator: StageInfo,
    /// The node witness generator data.
    pub node_witness_generator: StageInfo,
    /// The recursion tip data.
    pub recursion_tip_witness_generator: StageInfo,
    /// The scheduler data.
    pub scheduler_witness_generator: StageInfo,
    /// The compressor data.
    pub compressor: StageInfo,
}

pub async fn get_batches_data(
    batches: Vec<L1BatchNumber>,
    prover_db: &mut PoolConnection<Postgres>,
) -> eyre::Result<Vec<BatchData>> {
    let mut batches_data = Vec::new();
    for l1_batch_number in batches {
        let current_batch_data = BatchData {
            batch_number: l1_batch_number,
            basic_witness_generator: StageInfo::BasicWitnessGenerator {
                witness_generator_job_info: get_proof_basic_witness_generator_info_for_batch(
                    l1_batch_number,
                    prover_db,
                )
                .await?,
                prover_jobs_info: get_prover_jobs_info_for_batch(
                    l1_batch_number,
                    AggregationRound::BasicCircuits,
                    prover_db,
                )
                .await?,
            },
            leaf_witness_generator: StageInfo::LeafWitnessGenerator {
                witness_generator_jobs_info: get_proof_leaf_witness_generator_info_for_batch(
                    l1_batch_number,
                    prover_db,
                )
                .await?,
                prover_jobs_info: get_prover_jobs_info_for_batch(
                    l1_batch_number,
                    AggregationRound::LeafAggregation,
                    prover_db,
                )
                .await?,
            },
            node_witness_generator: StageInfo::NodeWitnessGenerator {
                witness_generator_jobs_info: get_proof_node_witness_generator_info_for_batch(
                    l1_batch_number,
                    prover_db,
                )
                .await?,
                prover_jobs_info: get_prover_jobs_info_for_batch(
                    l1_batch_number,
                    AggregationRound::NodeAggregation,
                    prover_db,
                )
                .await?,
            },
            recursion_tip_witness_generator: StageInfo::RecursionTipWitnessGenerator(
                get_proof_recursion_tip_witness_generator_info_for_batch(
                    l1_batch_number,
                    prover_db,
                )
                .await?,
            ),
            scheduler_witness_generator: StageInfo::SchedulerWitnessGenerator(
                get_proof_scheduler_witness_generator_info_for_batch(l1_batch_number, prover_db)
                    .await?,
            ),
            compressor: StageInfo::Compressor(
                get_proof_compression_job_info_for_batch(l1_batch_number, prover_db).await?,
            ),
        };
        batches_data.push(current_batch_data);
    }
    Ok(batches_data)
}

// Display functions

pub(crate) fn display_batch_status(batch_data: BatchData, flags: u32) {
    let stages = [
        (StageFlags::Bwg, batch_data.basic_witness_generator),
        (StageFlags::Lwg, batch_data.leaf_witness_generator),
        (StageFlags::Nwg, batch_data.node_witness_generator),
        (StageFlags::Rtwg, batch_data.recursion_tip_witness_generator),
        (StageFlags::Swg, batch_data.scheduler_witness_generator),
        (StageFlags::Compressor, batch_data.compressor),
    ];

    for (flag, stage) in stages {
        if flags == 0 || flags & flag.as_u32() != 0 {
            display_status_for_stage(stage);
        }
    }
}

fn display_status_for_stage(stage_info: StageInfo) {
    let max_attempts = 10;
    display_aggregation_round(&stage_info);
    let status = stage_info.witness_generator_jobs_status(max_attempts);
    match status {
        Status::Custom(msg) => {
            println!("{}: {} \n", stage_info.to_string().bold(), msg);
        }
        Status::Queued | Status::WaitingForProofs | Status::Stuck | Status::JobsNotFound => {
            println!("{}: {}", stage_info.to_string().bold(), status)
        }
        Status::InProgress | Status::Successful => {
            println!("{}: {}", stage_info.to_string().bold(), status);
            if let Some(job_status) = stage_info.prover_jobs_status(max_attempts) {
                println!("> {}: {}", "Prover Jobs".to_owned().bold(), job_status);
            }
        }
    }
}

#[allow(clippy::as_conversions, reason = "AggregationRound is an enum of u8s")]
fn display_aggregation_round(stage_info: &StageInfo) {
    if let Some(aggregation_round) = stage_info.aggregation_round() {
        println!(
            "\n-- {} --",
            format!("Aggregation Round {}", aggregation_round as u8).bold()
        );
    } else {
        println!("\n-- {} --", "Proof Compression".to_owned().bold());
    };
}

pub(crate) fn display_batch_info(batch_data: BatchData, flags: u32) -> eyre::Result<()> {
    let stages = [
        (StageFlags::Bwg, batch_data.basic_witness_generator),
        (StageFlags::Lwg, batch_data.leaf_witness_generator),
        (StageFlags::Nwg, batch_data.node_witness_generator),
        (StageFlags::Rtwg, batch_data.recursion_tip_witness_generator),
        (StageFlags::Swg, batch_data.scheduler_witness_generator),
        (StageFlags::Compressor, batch_data.compressor),
    ];

    for (flag, stage) in stages {
        if flags == 0 || flags & flag.as_u32() != 0 {
            display_info_for_stage(stage)?;
        }
    }
    Ok(())
}

fn display_info_for_stage(stage_info: StageInfo) -> eyre::Result<()> {
    let max_attempts = 10;
    display_aggregation_round(&stage_info);
    let status = stage_info.witness_generator_jobs_status(max_attempts);
    match status {
        Status::Custom(msg) => {
            println!("{}: {}", stage_info.to_string().bold(), msg);
        }
        Status::Queued | Status::WaitingForProofs | Status::JobsNotFound => {
            println!(" > {}: {}", stage_info.to_string().bold(), status)
        }
        Status::InProgress | Status::Stuck => {
            println!("v {}: {}", stage_info.to_string().bold(), status);
            match stage_info {
                StageInfo::BasicWitnessGenerator {
                    prover_jobs_info, ..
                } => {
                    display_prover_jobs_info(prover_jobs_info, max_attempts)?;
                }
                StageInfo::LeafWitnessGenerator {
                    witness_generator_jobs_info,
                    prover_jobs_info,
                } => {
                    display_leaf_witness_generator_jobs_info(
                        witness_generator_jobs_info,
                        max_attempts,
                    )?;
                    display_prover_jobs_info(prover_jobs_info, max_attempts)?;
                }
                StageInfo::NodeWitnessGenerator {
                    witness_generator_jobs_info,
                    prover_jobs_info,
                } => {
                    display_node_witness_generator_jobs_info(
                        witness_generator_jobs_info,
                        max_attempts,
                    )?;
                    display_prover_jobs_info(prover_jobs_info, max_attempts)?;
                }
                _ => (),
            }
        }
        Status::Successful => {
            println!("> {}: {}", stage_info.to_string().bold(), status);
            match stage_info {
                StageInfo::BasicWitnessGenerator {
                    prover_jobs_info, ..
                }
                | StageInfo::LeafWitnessGenerator {
                    prover_jobs_info, ..
                }
                | StageInfo::NodeWitnessGenerator {
                    prover_jobs_info, ..
                } => display_prover_jobs_info(prover_jobs_info, max_attempts)?,
                _ => (),
            }
        }
    }
    Ok(())
}

fn display_leaf_witness_generator_jobs_info(
    mut jobs_info: Vec<LeafWitnessGeneratorJobInfo>,
    max_attempts: u32,
) -> eyre::Result<()> {
    jobs_info.sort_by_key(|job| job._circuit_id);

    for job in jobs_info {
        println!(
            "   > {}: {}",
            format!(
                "{:?}",
                BaseLayerCircuitType::from_numeric_value(job._circuit_id.try_into()?)
            )
            .bold(),
            get_witness_generator_job_status(&job, max_attempts)
        )
    }
    Ok(())
}

fn display_node_witness_generator_jobs_info(
    mut jobs_info: Vec<NodeWitnessGeneratorJobInfo>,
    max_attempts: u32,
) -> eyre::Result<()> {
    jobs_info.sort_by_key(|job| job._circuit_id);

    for job in jobs_info {
        println!(
            "   > {}: {}",
            format!(
                "{:?}",
                BaseLayerCircuitType::from_numeric_value(job._circuit_id.try_into()?)
            )
            .bold(),
            get_witness_generator_job_status(&job, max_attempts)
        )
    }
    Ok(())
}

fn display_prover_jobs_info(
    prover_jobs_info: Vec<ProverJobFriInfo>,
    max_attempts: u32,
) -> eyre::Result<()> {
    let prover_jobs_status = get_prover_jobs_status_from_vec(&prover_jobs_info, max_attempts);

    if matches!(
        prover_jobs_status,
        Status::Successful | Status::JobsNotFound
    ) {
        println!(
            "> {}: {prover_jobs_status}",
            "Prover Jobs".to_owned().bold()
        );
        return Ok(());
    }

    println!(
        "v {}: {prover_jobs_status}",
        "Prover Jobs".to_owned().bold()
    );

    let mut jobs_by_circuit_id: BTreeMap<u32, Vec<ProverJobFriInfo>> = BTreeMap::new();
    prover_jobs_info.iter().for_each(|job| {
        jobs_by_circuit_id
            .entry(job._circuit_id)
            .or_default()
            .push(job.clone())
    });

    for (circuit_id, prover_jobs_info) in jobs_by_circuit_id {
        let status = get_prover_jobs_status_from_vec(&prover_jobs_info, max_attempts);
        println!(
            "   > {}: {}",
            format!(
                "{:?}",
                BaseLayerCircuitType::from_numeric_value(circuit_id.try_into()?)
            )
            .bold(),
            status
        );
        match status {
            Status::InProgress => display_job_status_count(prover_jobs_info),
            Status::Stuck => display_stuck_jobs(prover_jobs_info, max_attempts),
            _ => (),
        }
    }
    Ok(())
}

fn display_job_status_count(jobs: Vec<ProverJobFriInfo>) {
    let mut jobs_counts = ExtendedJobCountStatistics::default();
    jobs.iter().for_each(|job| match job._status {
        ProverJobStatus::Queued => jobs_counts.queued += 1,
        ProverJobStatus::InProgress(_) => jobs_counts.in_progress += 1,
        ProverJobStatus::Successful(_) => jobs_counts.successful += 1,
        ProverJobStatus::Failed(_) => jobs_counts.failed += 1,
        ProverJobStatus::Skipped | ProverJobStatus::Ignored | ProverJobStatus::InGPUProof => (),
    });

    println!("     - Total jobs: {}", jobs.len());
    println!("     - Successful: {}", jobs_counts.successful);
    println!("     - In Progress: {}", jobs_counts.in_progress);
    println!("     - Queued: {}", jobs_counts.queued);
    println!("     - Failed: {}", jobs_counts.failed);
}

fn display_stuck_jobs(jobs: Vec<ProverJobFriInfo>, max_attempts: u32) {
    jobs.iter().for_each(|job| {
        if matches!(
            get_prover_job_status(job.clone(), max_attempts),
            Status::Stuck
        ) {
            println!(
                "     - Prover Job: {} stuck after {} attempts",
                job._id, job._attempts
            );
        }
    })
}
