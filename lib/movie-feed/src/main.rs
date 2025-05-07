mod config;

use tmdb::Tmdb;
use tmdb::endpoints::person::combined_credits;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::format::FmtSpan;
use crate::config::config;

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
    
    let tmbd = Tmdb::new(http_client, config.tmdb_token.clone());

    let person_id = "19498"; // Jon Bernthal

    let kek = combined_credits::get(&tmbd, person_id).await;
    dbg!(kek.unwrap());
}
