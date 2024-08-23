use crate::utils::db::{
    queries::{get_batch_proofs_stuck_at_prover_in_agg_round, get_batch_proofs_stuck_at_wg_round},
    types::{
        BasicWitnessGeneratorJobInfo, LeafWitnessGeneratorJobInfo, NodeWitnessGeneratorJobInfo,
        RecursionTipWitnessGeneratorJobInfo, SchedulerWitnessGeneratorJobInfo,
    },
};
use itertools::Itertools;
use spinoff::{spinners::Dots, Color, Spinner};
use sqlx::{pool::PoolConnection, postgres::PgRow, FromRow, Postgres};
use zksync_ethers_rs::types::zksync::{basic_fri_types::AggregationRound, L1BatchNumber};

pub async fn find_map_stuck_wg_jobs_in_aggregation_round<WG>(
    aggregation_round: AggregationRound,
    map: impl Fn(Vec<WG>) -> Vec<L1BatchNumber>,
    prover_db: &mut PoolConnection<Postgres>,
) -> eyre::Result<()>
where
    WG: for<'row> FromRow<'row, PgRow>,
{
    let mut spinner = Spinner::new(
        Dots,
        format!("Searching for stuck witness generator jobs in {aggregation_round}"),
        Color::Blue,
    );
    let stuck_jobs: Vec<WG> =
        get_batch_proofs_stuck_at_wg_round(aggregation_round, prover_db).await?;
    if !stuck_jobs.is_empty() {
        spinner.fail(&format!(
            "Stuck witness generator jobs found in {aggregation_round}: {:?}",
            map(stuck_jobs)
        ));
    } else {
        spinner.success(&format!(
            "No stuck witness generator jobs found in {aggregation_round}"
        ));
    }
    Ok(())
}

pub fn map_bwg_info(a: Vec<BasicWitnessGeneratorJobInfo>) -> Vec<L1BatchNumber> {
    a.iter().map(|job| job.l1_batch_number).collect::<Vec<_>>()
}

pub fn map_leaf_wg_info(a: Vec<LeafWitnessGeneratorJobInfo>) -> Vec<L1BatchNumber> {
    a.iter()
        .into_group_map_by(|job| job.l1_batch_number)
        .keys()
        .cloned()
        .collect()
}

pub fn map_node_wg_info(a: Vec<NodeWitnessGeneratorJobInfo>) -> Vec<L1BatchNumber> {
    a.iter()
        .into_group_map_by(|job| job.l1_batch_number)
        .keys()
        .cloned()
        .collect()
}

pub fn map_recursion_tip_wg_info(
    a: Vec<RecursionTipWitnessGeneratorJobInfo>,
) -> Vec<L1BatchNumber> {
    a.iter().map(|job| job.l1_batch_number).collect::<Vec<_>>()
}

pub fn map_scheduler_wg_info(a: Vec<SchedulerWitnessGeneratorJobInfo>) -> Vec<L1BatchNumber> {
    a.iter().map(|job| job.l1_batch_number).collect::<Vec<_>>()
}

pub async fn find_stuck_prover_jobs_in_aggregation_round(
    aggregation_round: AggregationRound,
    prover_db: &mut PoolConnection<Postgres>,
) -> eyre::Result<()> {
    let mut spinner = Spinner::new(
        Dots,
        format!("Searching for stuck proofs in {aggregation_round}"),
        Color::Blue,
    );
    let stuck_prover_jobs =
        get_batch_proofs_stuck_at_prover_in_agg_round(prover_db, AggregationRound::NodeAggregation)
            .await?;
    if !stuck_prover_jobs.is_empty() {
        let stuck_batch_proofs_in_prover = stuck_prover_jobs
            .iter()
            .into_group_map_by(|job| job.l1_batch_number);
        spinner.fail(&format!(
            "Stuck proofs found in {aggregation_round}: {:?}",
            stuck_batch_proofs_in_prover.keys().collect::<Vec<_>>()
        ));
    } else {
        spinner.success(&format!("No stuck proofs found in {aggregation_round}"));
    }
    Ok(())
}
