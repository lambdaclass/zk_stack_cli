use zks::{
    cli::{self},
    config::load_config,
};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::ERROR)
        .init();

    let config = match load_config() {
        Ok(config) => config,
        Err(err) => {
            tracing::error!("{err:?}");
            std::process::exit(1);
        }
    };

    match cli::start(config).await {
        Ok(_) => {}
        Err(err) => {
            tracing::error!("{err:?}");
            std::process::exit(1);
        }
    }
}
