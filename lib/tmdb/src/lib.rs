pub mod endpoints;
pub mod models;

use reqwest::Client;
use secrecy::SecretString;

pub const API_HOST: &str = "api.themoviedb.org";
pub const API_VERSION: u8 = 3;
pub const SITE_URL: &str = "https://www.themoviedb.org";

pub struct Tmdb {
    token: SecretString,
    http_client: Client,
}

impl Tmdb {
    pub fn new(http_client: Client, token: SecretString) -> Self {
        Self { token, http_client }
    }
}

#[cfg(any(test, feature = "test_utils"))]
impl Default for Tmdb {
    fn default() -> Self {
        Self {
            token: SecretString::from("THIS_IS_A_TEST"),
            http_client: Client::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use secrecy::ExposeSecret;
    use std::any::{Any, TypeId};

    #[test]
    fn test_new() {
        let tmdb = Tmdb::new(Client::new(), SecretString::from("NO_TOKEN_REQUIRED"));

        assert_eq!(tmdb.token.expose_secret(), "NO_TOKEN_REQUIRED");
        assert_eq!(tmdb.http_client.type_id(), TypeId::of::<Client>());
    }

    #[test]
    fn test_default() {
        let tmdb = Tmdb::default();

        assert_eq!(tmdb.token.expose_secret(), "THIS_IS_A_TEST");
        assert_eq!(tmdb.http_client.type_id(), TypeId::of::<Client>());
    }
}
