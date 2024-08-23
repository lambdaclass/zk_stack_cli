use crate::config::ZKSyncConfig;
use clap::Subcommand;
use eyre::ContextCompat;
use itertools::Itertools;
use sqlx::{pool::PoolConnection, postgres::PgRow, Executor, FromRow, Postgres};
use zksync_ethers_rs::types::zksync::basic_fri_types::AggregationRound;

use super::types::{
    BasicWitnessGeneratorJobInfo, LeafWitnessGeneratorJobInfo, NodeWitnessGeneratorJobInfo,
    ProverJobFriInfo, RecursionTipWitnessGeneratorJobInfo, SchedulerWitnessGeneratorJobInfo,
};

#[derive(Subcommand)]
pub(crate) enum Command {
    #[clap(about = "List all the stuck batch proofs.", visible_alias = "stuck")]
    StuckBatchProofs,
}

pub const CURRENT_MAX_ATTEMPTS: i32 = 10;

impl Command {
    pub async fn run(self, cfg: ZKSyncConfig) -> eyre::Result<()> {
        let mut prover_db = cfg
            .db
            .context("DB config missing")?
            .prover
            .acquire()
            .await?;
        match self {
            Command::StuckBatchProofs => {
                let mut found_stuck_batches = false;
                let mut aggregation_round = AggregationRound::BasicCircuits;
                let stuck_jobs: Vec<BasicWitnessGeneratorJobInfo> =
                    get_batch_proofs_stuck_at_wg_round(aggregation_round, &mut prover_db).await?;
                if !stuck_jobs.is_empty() {
                    found_stuck_batches = true;
                    println!(
                        "Stuck batch jobs in {aggregation_round:?}: {:?}",
                        stuck_jobs
                            .iter()
                            .map(|job| job.l1_batch_number)
                            .collect::<Vec<_>>()
                    );
                }
                let any_stuck_proof =
                    find_stuck_prover_jobs_in_aggregation_round(aggregation_round, &mut prover_db)
                        .await?;

                found_stuck_batches |= any_stuck_proof;

                aggregation_round = AggregationRound::LeafAggregation;
                let stuck_jobs: Vec<LeafWitnessGeneratorJobInfo> =
                    get_batch_proofs_stuck_at_wg_round(aggregation_round, &mut prover_db).await?;
                if !stuck_jobs.is_empty() {
                    found_stuck_batches = true;
                    let stuck_batch_proofs_in_wg = stuck_jobs
                        .iter()
                        .into_group_map_by(|job| job.l1_batch_number);
                    println!(
                        "Stuck batch jobs in {aggregation_round:?}: {:?}",
                        stuck_batch_proofs_in_wg.keys().collect::<Vec<_>>()
                    );
                }
                let any_stuck_proof =
                    find_stuck_prover_jobs_in_aggregation_round(aggregation_round, &mut prover_db)
                        .await?;

                found_stuck_batches |= any_stuck_proof;

                aggregation_round = AggregationRound::NodeAggregation;
                let stuck_jobs: Vec<NodeWitnessGeneratorJobInfo> =
                    get_batch_proofs_stuck_at_wg_round(aggregation_round, &mut prover_db).await?;
                if !stuck_jobs.is_empty() {
                    found_stuck_batches = true;
                    let stuck_batch_proofs_in_wg = stuck_jobs
                        .iter()
                        .into_group_map_by(|job| job.l1_batch_number);
                    println!(
                        "Stuck batch jobs in {aggregation_round:?}: {:?}",
                        stuck_batch_proofs_in_wg.keys().collect::<Vec<_>>()
                    );
                }
                let any_stuck_proof =
                    find_stuck_prover_jobs_in_aggregation_round(aggregation_round, &mut prover_db)
                        .await?;

                found_stuck_batches |= any_stuck_proof;

                aggregation_round = AggregationRound::RecursionTip;
                let stuck_jobs: Vec<RecursionTipWitnessGeneratorJobInfo> =
                    get_batch_proofs_stuck_at_wg_round(aggregation_round, &mut prover_db).await?;
                if !stuck_jobs.is_empty() {
                    found_stuck_batches = true;
                    let stuck_batch_proofs_in_wg = stuck_jobs
                        .iter()
                        .into_group_map_by(|job| job.l1_batch_number);
                    println!(
                        "Stuck batch jobs in {aggregation_round:?}: {:?}",
                        stuck_batch_proofs_in_wg.keys().collect::<Vec<_>>()
                    );
                }
                let any_stuck_proof =
                    find_stuck_prover_jobs_in_aggregation_round(aggregation_round, &mut prover_db)
                        .await?;

                found_stuck_batches |= any_stuck_proof;

                aggregation_round = AggregationRound::Scheduler;
                let stuck_jobs: Vec<SchedulerWitnessGeneratorJobInfo> =
                    get_batch_proofs_stuck_at_wg_round(aggregation_round, &mut prover_db).await?;
                if !stuck_jobs.is_empty() {
                    found_stuck_batches = true;
                    let stuck_batch_proofs_in_wg = stuck_jobs
                        .iter()
                        .into_group_map_by(|job| job.l1_batch_number);
                    println!(
                        "Stuck batch jobs in {aggregation_round:?}: {:?}",
                        stuck_batch_proofs_in_wg.keys().collect::<Vec<_>>()
                    );
                }
                let any_stuck_proof =
                    find_stuck_prover_jobs_in_aggregation_round(aggregation_round, &mut prover_db)
                        .await?;

                found_stuck_batches |= any_stuck_proof;

                if !found_stuck_batches {
                    println!("No stuck batch proofs found.");
                }
            }
        };

        Ok(())
    }
}

async fn find_stuck_prover_jobs_in_aggregation_round(
    aggregation_round: AggregationRound,
    prover_db: &mut PoolConnection<Postgres>,
) -> eyre::Result<bool> {
    let mut any_stuck_proof = false;
    let stuck_prover_jobs =
        get_batch_proofs_stuck_at_prover_in_agg_round(prover_db, AggregationRound::NodeAggregation)
            .await?;
    if !stuck_prover_jobs.is_empty() {
        any_stuck_proof = true;
        let stuck_batch_proofs_in_prover_agg_round_0 = stuck_prover_jobs
            .iter()
            .into_group_map_by(|job| job.l1_batch_number);
        println!(
            "Stuck batch proofs in ({aggregation_round:?}): {:?}",
            stuck_batch_proofs_in_prover_agg_round_0
                .keys()
                .collect::<Vec<_>>()
        );
    }
    Ok(any_stuck_proof)
}

async fn get_batch_proofs_stuck_at_wg_round<WG>(
    aggregation_round: AggregationRound,
    prover_db: &mut PoolConnection<Postgres>,
) -> eyre::Result<Vec<WG>>
where
    WG: for<'row> FromRow<'row, PgRow>,
{
    let table = input_table_name_for(aggregation_round);
    let query = format!(
        "
        SELECT l1_batch_number
        FROM {table}
        WHERE
            attempts = {CURRENT_MAX_ATTEMPTS}
            AND status != 'success'
        "
    );
    prover_db
        .fetch_all(query.as_str())
        .await?
        .iter()
        .map(WG::from_row)
        .collect::<Result<Vec<WG>, _>>()
        .map_err(Into::into)
}

async fn get_batch_proofs_stuck_at_prover_in_agg_round(
    prover_db: &mut PoolConnection<Postgres>,
    aggregation_round: AggregationRound,
) -> eyre::Result<Vec<ProverJobFriInfo>> {
    let query = format!(
        "
        SELECT l1_batch_number
        FROM prover_jobs_fri
        WHERE
            attempts = {CURRENT_MAX_ATTEMPTS}
            AND status != 'success'
            AND aggregation_round = {}
        ",
        aggregation_round as u8
    );
    prover_db
        .fetch_all(query.as_str())
        .await?
        .iter()
        .map(ProverJobFriInfo::from_row)
        .collect::<Result<Vec<ProverJobFriInfo>, _>>()
        .map_err(Into::into)
}

fn input_table_name_for(aggregation_round: AggregationRound) -> &'static str {
    match aggregation_round {
        AggregationRound::BasicCircuits => "witness_inputs_fri",
        AggregationRound::LeafAggregation => "leaf_aggregation_witness_jobs_fri",
        AggregationRound::NodeAggregation => "node_aggregation_witness_jobs_fri",
        AggregationRound::RecursionTip => "recursion_tip_witness_jobs_fri",
        AggregationRound::Scheduler => "scheduler_witness_jobs_fri",
    }
}
