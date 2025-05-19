use crate::api_version::ApiVersion;
use crate::{DEFAULT_API_URL, Tmdb};
use reqwest::{Method, Response};
use std::fmt::Display;
use url::ParseError;

pub mod v3;

#[derive(thiserror::Error, Debug)]
pub enum RequestError {
    #[error("reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("url parsing error: {0}")]
    UrlParseError(#[from] ParseError),
}

pub(crate) async fn request<P: AsRef<str> + Display>(
    tmdb: &Tmdb,
    path: P,
    method: Method,
) -> Result<Response, RequestError> {
    use secrecy::ExposeSecret;

    let url = tmdb
        .api_url
        .join(ApiVersion::V3.base_path())
        .and_then(|url| url.join(path.as_ref()))?;

    if url.as_str().starts_with(DEFAULT_API_URL.as_str()) {
        panic!("REMOTE REQUEST {url}"); // TODO - REMOVE
    }

    tmdb.http_client
        .request(method, url)
        .bearer_auth(tmdb.token.expose_secret())
        .send()
        .await
        .map_err(RequestError::Reqwest)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use http::StatusCode;
    use tmdb_test_utils::api::misc::status_codes::mock_get_ok;
    use tmdb_test_utils::mockito::{Mock, ServerGuard};
    use tmdb_test_utils::start_mock_tmdb_api;

    async fn init() -> (Tmdb, ServerGuard, Mock, String) {
        let mut server = start_mock_tmdb_api().await;
        let (mock, path) = mock_get_ok(&mut server).await;

        let mut tmdb = Tmdb::default();
        tmdb.override_api_url(server.url().as_str()).unwrap();

        (tmdb, server, mock, path)
    }

    #[tokio::test]
    async fn test_request() {
        let (tmdb, _server, mock, path) = init().await;

        let response = request(&tmdb, path, Method::GET).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        mock.assert();
    }
}
