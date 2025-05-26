use crate::Tmdb;
use crate::api_version::ApiVersion;
use crate::models::v3::tmdb_error::{TmdbError, UnknownTmdbError};
use reqwest::{Method, Response};
use std::fmt::Display;
use url::ParseError;

pub mod v3;

#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum RequestError {
    #[error("reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("url parsing error: {0}")]
    UrlParseError(#[from] ParseError),
    #[error("tmdb error: {0}")]
    TmdbError(#[from] TmdbError),
    #[error("unknown tmdb error: {0}")]
    UnknownTmdbError(#[from] UnknownTmdbError),
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
    use tmdb_test_utils::api::v3::errors::mock_invalid_id;
    use tmdb_test_utils::mockito::ServerGuard;
    use tmdb_test_utils::start_mock_tmdb_api;

    async fn init() -> (Tmdb, ServerGuard) {
        let server = start_mock_tmdb_api().await;

        let mut tmdb = Tmdb::default();
        tmdb.override_api_url(server.url().as_str()).unwrap();

        (tmdb, server)
    }

    #[tokio::test]
    async fn test_request() {
        let (tmdb, mut server) = init().await;
        let (mock, path) = mock_get_ok(&mut server).await;

        let response = request(&tmdb, path, Method::GET).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        mock.assert();
    }

    #[tokio::test]
    async fn test_request_tmdb_invalid_id_error() {
        let (tmdb, mut server) = init().await;
        let (mock, path) = mock_invalid_id(&mut server).await;

        let response = request(&tmdb, path, Method::GET).await.unwrap();
        let error = TmdbError::try_from_response(response).await;

        assert_eq!(error, Ok(TmdbError::InvalidId));
        mock.assert();
    }
}
