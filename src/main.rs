use zksync_era_cli::cli;

#[tokio::main]
async fn main() {
    env_logger::builder()
        .filter_module("reqwest::connect", log::LevelFilter::Off)
        .filter_module("rustls::client", log::LevelFilter::Off)
        .filter_module("rustls::common_state", log::LevelFilter::Off)
        .filter_level(log::LevelFilter::Debug)
        .init();

    cli::start().await.unwrap();
}
