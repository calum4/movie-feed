mod api;
mod config;

use crate::api::{ApiState, start_api_server};
use crate::config::config;
use tmdb::Tmdb;
use tracing::error;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::format::FmtSpan;

#[cfg(debug_assertions)]
fn start_tracing() {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_file(true)
        .with_line_number(true)
        .with_env_filter(EnvFilter::from_default_env())
        .with_span_events(FmtSpan::CLOSE)
        .init();
}

#[cfg(not(debug_assertions))]
fn start_tracing() {
    tracing_subscriber::fmt()
        .with_target(true)
        .with_file(false)
        .with_line_number(false)
        .with_env_filter(EnvFilter::from_default_env())
        .with_span_events(FmtSpan::CLOSE)
        .init();
}

#[tokio::main]
async fn main() {
    let config = config();

    start_tracing();

    let http_client = reqwest::Client::new();
    let tmdb = Tmdb::new(http_client, config.tmdb_token.clone());
    let api_state = ApiState::new(tmdb);

    let handle = match start_api_server(config, api_state).await {
        Ok(handle) => handle,
        Err(error) => {
            error!("unable to start api server, exiting! {error}");
            return;
        }
    };

    if let Err(error) = handle.await {
        error!("there was an issue with the api server, exiting: {error}");
        return;
    }
}
