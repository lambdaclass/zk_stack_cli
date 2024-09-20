use colored::Colorize;
use spinoff::{spinners, Color, Spinner};
use zksync_ethers_rs::{
    providers::{Http, Provider},
    types::zksync::L1BatchNumber,
    ZKMiddleware,
};

pub(crate) async fn display_batches_proof_time_from_l1_batch_details(
    batches: Vec<L1BatchNumber>,
    current_batch: L1BatchNumber,
    l2_provider: Provider<Http>,
) -> eyre::Result<()> {
    let msg = if batches.len() > 1 {
        "Fetching Batches' Data"
    } else {
        "Fetching Batch's Data"
    };

    let mut spinner = Spinner::new(spinners::Dots, msg, Color::Blue);

    let mut batches_details = Vec::new();
    for batch in batches {
        if batch.0 > current_batch.0 {
            println!("Batch doesn't exist, Current batch: {}", current_batch.0);
            break;
        }
        batches_details.push(l2_provider.get_l1_batch_details(batch.0).await?);
    }
    spinner.success("Data Retrieved");
    for batch_details in batches_details {
        println!(
            "{} {} {}",
            "=".repeat(8),
            format!("Batch {:0>5} Status", batch_details.number.0)
                .bold()
                .bright_cyan()
                .on_black(),
            "=".repeat(8)
        );
        if let Some(committed_at) = batch_details.base.committed_at {
            println!("{}: {committed_at}", "Committed At".yellow());
        }
        if let Some(commit_tx_hash) = batch_details.base.commit_tx_hash {
            println!(
                "Commit Tx Hash: {}",
                format!("{commit_tx_hash:?}").bright_blue()
            );
        }
        if let Some(proven_at) = batch_details.base.proven_at {
            println!("{}: {proven_at}", "Proven At".yellow());
        }
        if let Some(prove_tx_hash) = batch_details.base.prove_tx_hash {
            println!(
                "Proven Tx Hash: {}",
                format!("{prove_tx_hash:?}").bright_blue()
            );
        }
        if let (Some(committed_at), Some(proven_at)) = (
            batch_details.base.committed_at,
            batch_details.base.proven_at,
        ) {
            let duration = proven_at - committed_at;
            let formatted_duration = format!(
                "{:02}:{:02}:{:02}",
                duration.num_hours(),
                duration.num_minutes() % 60,
                duration.num_seconds() % 60
            );
            println!(
                "ProofTime from Committed to Proven: {}",
                formatted_duration.on_black().green()
            );
        }
    }
    Ok(())
}

pub(crate) async fn display_batches_details(
    batches: Vec<L1BatchNumber>,
    current_batch: L1BatchNumber,
    l2_provider: Provider<Http>,
) -> eyre::Result<()> {
    let msg = if batches.len() > 1 {
        "Fetching Batches' Data"
    } else {
        "Fetching Batch's Data"
    };

    let mut spinner = Spinner::new(spinners::Dots, msg, Color::Blue);

    let mut batches_details = Vec::new();
    for batch in batches {
        if batch.0 > current_batch.0 {
            println!("Batch doesn't exist, Current batch: {}", current_batch.0);
            break;
        }
        batches_details.push(l2_provider.get_l1_batch_details(batch.0).await?);
    }
    spinner.success("Data Retrieved");
    for batch_details in batches_details {
        println!(
            "{} {} {}",
            "=".repeat(8),
            format!("Batch {:0>5} Status", batch_details.number.0)
                .bold()
                .bright_cyan()
                .on_black(),
            "=".repeat(8)
        );

        println!("{batch_details:#?}");
    }
    Ok(())
}
