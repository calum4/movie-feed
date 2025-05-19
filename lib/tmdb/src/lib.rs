pub mod api_version;
pub mod endpoints;
pub mod models;

use reqwest::Client;
use secrecy::SecretString;
use std::sync::LazyLock;
use url::Url;

static DEFAULT_API_URL: LazyLock<Url> =
    LazyLock::new(|| Url::parse("https://api.themoviedb.org/").expect("valid str, tested"));
static SITE_URL: LazyLock<Url> =
    LazyLock::new(|| Url::parse("https://www.themoviedb.org/").expect("valid str, tested"));
static IMDB_SITE_URL: LazyLock<Url> =
    LazyLock::new(|| Url::parse("https://www.imdb.com/").expect("valid str, tested"));

pub struct Tmdb {
    token: SecretString,
    http_client: Client,
    api_url: Url,
}

impl Tmdb {
    pub fn new(http_client: Client, token: SecretString) -> Self {
        Self {
            token,
            http_client,
            api_url: DEFAULT_API_URL.clone(),
        }
    }

    pub fn override_api_url<U: TryInto<Url>>(&mut self, url: U) -> Result<(), U::Error> {
        let url = url.try_into()?;
        self.api_url = url;
        Ok(())
    }
}

#[cfg(any(test, feature = "test_utils"))]
impl Default for Tmdb {
    fn default() -> Self {
        Self::new(Client::new(), SecretString::from("THIS_IS_A_TEST"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use secrecy::ExposeSecret;
    use std::any::{Any, TypeId};

    #[test]
    fn test_static_default_api_url() {
        assert_eq!(DEFAULT_API_URL.as_str(), "https://api.themoviedb.org/");
    }

    #[test]
    fn test_static_site_url() {
        assert_eq!(SITE_URL.as_str(), "https://www.themoviedb.org/");
    }

    #[test]
    fn test_static_imdb_site_url() {
        assert_eq!(IMDB_SITE_URL.as_str(), "https://www.imdb.com/");
    }

    #[test]
    fn test_new() {
        let tmdb = Tmdb::new(Client::new(), SecretString::from("NO_TOKEN_REQUIRED"));

        assert_eq!(tmdb.token.expose_secret(), "NO_TOKEN_REQUIRED");
        assert_eq!(tmdb.http_client.type_id(), TypeId::of::<Client>());
        assert_eq!(tmdb.api_url.as_str(), DEFAULT_API_URL.as_str());
    }

    #[test]
    fn test_default() {
        let tmdb = Tmdb::default();

        assert_eq!(tmdb.token.expose_secret(), "THIS_IS_A_TEST");
        assert_eq!(tmdb.http_client.type_id(), TypeId::of::<Client>());
        assert_eq!(tmdb.api_url.as_str(), DEFAULT_API_URL.as_str());
    }

    #[test]
    fn test_override_api_url() {
        let mut tmdb = Tmdb::default();

        tmdb.override_api_url("https://example.com/").unwrap();
        assert_eq!(tmdb.api_url.as_str(), "https://example.com/");

        assert_eq!(
            tmdb.override_api_url("INVALID_URL"),
            Err(url::ParseError::RelativeUrlWithoutBase)
        );
    }
}
