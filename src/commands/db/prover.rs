use crate::{
    config::ZKSyncConfig,
    utils::db::prover::{
        find_map_stuck_wg_jobs_in_aggregation_round, find_stuck_prover_jobs_in_aggregation_round,
        map_bwg_info, map_leaf_wg_info, map_node_wg_info, map_recursion_tip_wg_info,
        map_scheduler_wg_info,
    },
};
use clap::Subcommand;
use eyre::ContextCompat;
use zksync_ethers_rs::types::zksync::basic_fri_types::AggregationRound;

#[derive(Subcommand)]
pub(crate) enum Command {
    #[clap(about = "List all the stuck batch proofs.", visible_alias = "stuck")]
    StuckBatchProofs,
}

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
                let mut aggregation_round = AggregationRound::BasicCircuits;
                let any_stuck_basic_wg_job = find_map_stuck_wg_jobs_in_aggregation_round(
                    aggregation_round,
                    map_bwg_info,
                    &mut prover_db,
                )
                .await?;
                let any_stuck_basic_proof =
                    find_stuck_prover_jobs_in_aggregation_round(aggregation_round, &mut prover_db)
                        .await?;

                aggregation_round = AggregationRound::LeafAggregation;
                let any_stuck_leaf_wg_job = find_map_stuck_wg_jobs_in_aggregation_round(
                    aggregation_round,
                    map_leaf_wg_info,
                    &mut prover_db,
                )
                .await?;
                let any_stuck_leaf_proof =
                    find_stuck_prover_jobs_in_aggregation_round(aggregation_round, &mut prover_db)
                        .await?;

                aggregation_round = AggregationRound::NodeAggregation;
                let any_stuck_node_wg_job = find_map_stuck_wg_jobs_in_aggregation_round(
                    aggregation_round,
                    map_node_wg_info,
                    &mut prover_db,
                )
                .await?;
                let any_stuck_node_proof =
                    find_stuck_prover_jobs_in_aggregation_round(aggregation_round, &mut prover_db)
                        .await?;

                aggregation_round = AggregationRound::RecursionTip;
                let any_stuck_recursion_tip_wg_job = find_map_stuck_wg_jobs_in_aggregation_round(
                    aggregation_round,
                    map_recursion_tip_wg_info,
                    &mut prover_db,
                )
                .await?;
                let any_stuck_recursion_tip_proof =
                    find_stuck_prover_jobs_in_aggregation_round(aggregation_round, &mut prover_db)
                        .await?;

                aggregation_round = AggregationRound::Scheduler;
                let any_stuck_scheduler_wg_job = find_map_stuck_wg_jobs_in_aggregation_round(
                    aggregation_round,
                    map_scheduler_wg_info,
                    &mut prover_db,
                )
                .await?;
                let any_stuck_scheduler_proof =
                    find_stuck_prover_jobs_in_aggregation_round(aggregation_round, &mut prover_db)
                        .await?;

                if !any_stuck_basic_wg_job
                    && !any_stuck_basic_proof
                    && !any_stuck_leaf_wg_job
                    && !any_stuck_leaf_proof
                    && !any_stuck_node_wg_job
                    && !any_stuck_node_proof
                    && !any_stuck_recursion_tip_wg_job
                    && !any_stuck_recursion_tip_proof
                    && !any_stuck_scheduler_wg_job
                    && !any_stuck_scheduler_proof
                {
                    println!("No stuck batch proofs found in any aggregation round");
                }
            }
        };

        Ok(())
    }
}
