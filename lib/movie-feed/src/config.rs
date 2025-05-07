use std::fs::read_to_string;
use std::sync::OnceLock;
use figment::Figment;
use figment::providers::Env;
use secrecy::{SecretString};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct Config {
    pub(crate) tmdb_token: SecretString,
}

pub(crate) fn config() -> &'static Config {
    static CONFIG: OnceLock<Config> = OnceLock::new();

    // TODO - accept config from cli with clap
    CONFIG.get_or_init(env_config)
}

#[derive(Debug, Deserialize)]
struct EnvConfig {
    tmdb_token: Option<SecretString>,
    tmdb_token_file: Option<String>,
}

fn env_config() -> Config {
    let mut config: EnvConfig = Figment::new()
        .merge(Env::prefixed("MOVIE_FEED_"))
        .extract()
        .expect("unable to construct environment config");

    if let Some(tmdb_token_file) = &config.tmdb_token_file {
        let token = read_to_string(tmdb_token_file).expect("unable to read tmdb token file with the path provided in the MOVIE_FEED_TMDB_TOKEN_FILE environment variable");
        
        config.tmdb_token = Some(token.into());
    }
    
    Config {
        tmdb_token: config.tmdb_token.expect("missing tmdb_token field"),
    }
}

// TODO - Tests
