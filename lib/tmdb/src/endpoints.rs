use crate::{API_HOST, API_VERSION, Tmdb};
use reqwest::{Method, Response};
use std::fmt::Display;

pub mod person;

#[cfg(not(test))]
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

#[cfg(test)]
pub(crate) async fn request<P: AsRef<str> + Display>(
    _tmdb: &Tmdb,
    path: P,
    method: Method,
) -> Result<Response, reqwest::Error> {
    use http::response::Builder;
    use reqwest::{ResponseBuilderExt, Url};
    use std::fs::read_to_string;

    let url = Url::parse(format!("https://{API_HOST}/{API_VERSION}/{path}").as_str()).unwrap();

    let body = read_to_string(format!(
        "response_files{}/{}.json",
        url.path(),
        method.as_str()
    ))
    .unwrap();

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
    use reqwest::Client;

    fn init() -> Tmdb {
        Tmdb::new(Client::new(), "NO_TOKEN_REQUIRED".into())
    }

    #[tokio::test]
    async fn test_request() {
        let tmdb = init();

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
    }
}
