use crate::{API_HOST, API_VERSION, Tmdb};
use reqwest::{Method, Response};
use std::fmt::Display;

pub mod v3;

#[cfg(not(any(test, feature = "use_prebaked_responses")))]
pub(crate) async fn request<P: AsRef<str> + Display>(
    tmdb: &Tmdb,
    path: P,
    method: Method,
) -> Result<Response, reqwest::Error> {
    use secrecy::ExposeSecret;

    let url = format!("https://{API_HOST}/{API_VERSION}/{path}");

    tmdb.http_client
        .request(method, url)
        .bearer_auth(tmdb.token.expose_secret())
        .send()
        .await
}

#[cfg(any(test, feature = "use_prebaked_responses"))]
pub(crate) async fn request<P: AsRef<str> + Display>(
    _tmdb: &Tmdb,
    path: P,
    method: Method,
) -> Result<Response, reqwest::Error> {
    use http::response::Builder;
    use reqwest::{ResponseBuilderExt, Url};
    use std::fs::read_to_string;
    use tracing::warn;

    const MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");

    warn!("using pre-baked responses!");

    let url = Url::parse(format!("https://{API_HOST}/{API_VERSION}/{path}").as_str()).unwrap();
    let path = format!(
        "{MANIFEST_DIR}/response_files{}/{}.json",
        url.path(),
        method.as_str()
    );

    let body = match read_to_string(path.as_str()) {
        Ok(body) => body,
        Err(error) => {
            panic!(r#"unable to open response file with path "{path}", error: {error}"#);
        }
    };

    let response = Builder::new()
        .status(200)
        .url(url.clone())
        .body(body)
        .unwrap();

    Ok(Response::from(response))
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use tracing_test::traced_test;

    #[tokio::test]
    #[traced_test]
    async fn test_request() {
        let tmdb = Tmdb::default();

        #[derive(serde::Deserialize)]
        struct Body {
            filename: String,
            date: String,
        }

        let body: Body = request(&tmdb, "tests/", Method::GET)
            .await
            .unwrap()
            .json()
            .await
            .unwrap();

        assert_eq!(body.filename.as_str(), "GET.json");
        assert_eq!(body.date.as_str(), "2025-05-06");
        assert!(logs_contain("using pre-baked responses!"));
    }
}
