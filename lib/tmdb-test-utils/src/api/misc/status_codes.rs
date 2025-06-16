use http::{Method, StatusCode};
use mockito::{Mock, ServerGuard};

pub async fn mock_status_code(
    server: &mut ServerGuard,
    method: Method,
    status_code: StatusCode,
) -> (Mock, String) {
    let path = format!("/status_codes/{}", status_code.as_u16());

    (
        mock_status_code_at_path(server, path.as_str(), method, status_code).await,
        path,
    )
}

pub async fn mock_status_code_at_path(
    server: &mut ServerGuard,
    path: &str,
    method: Method,
    status_code: StatusCode,
) -> Mock {
    server
        .mock(method.as_str(), path)
        .with_status(status_code.as_u16() as usize)
        .create_async()
        .await
}

pub async fn mock_get_ok(server: &mut ServerGuard) -> (Mock, String) {
    mock_status_code(server, Method::GET, StatusCode::OK).await
}

pub async fn mock_get_bad_request(server: &mut ServerGuard) -> (Mock, String) {
    mock_status_code(server, Method::GET, StatusCode::BAD_REQUEST).await
}

pub async fn mock_get_unauthorized(server: &mut ServerGuard) -> (Mock, String) {
    mock_status_code(server, Method::GET, StatusCode::UNAUTHORIZED).await
}

pub async fn mock_get_forbidden(server: &mut ServerGuard) -> (Mock, String) {
    mock_status_code(server, Method::GET, StatusCode::FORBIDDEN).await
}

pub async fn mock_get_not_found(server: &mut ServerGuard) -> (Mock, String) {
    mock_status_code(server, Method::GET, StatusCode::NOT_FOUND).await
}

pub async fn mock_get_too_many_requests(server: &mut ServerGuard) -> (Mock, String) {
    mock_status_code(server, Method::GET, StatusCode::TOO_MANY_REQUESTS).await
}

pub async fn mock_get_internal_server_error(server: &mut ServerGuard) -> (Mock, String) {
    mock_status_code(server, Method::GET, StatusCode::INTERNAL_SERVER_ERROR).await
}

pub async fn mock_get_not_implemented(server: &mut ServerGuard) -> (Mock, String) {
    mock_status_code(server, Method::GET, StatusCode::NOT_IMPLEMENTED).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::start_mock_tmdb_api;
    use http::StatusCode;
    use reqwest::Client;

    #[tokio::test]
    async fn test_mock_status_code_at_path() {
        let mut server = start_mock_tmdb_api().await;
        let path = "/test_mock_status_code_at_path/POST/400";

        let mock =
            mock_status_code_at_path(&mut server, path, Method::POST, StatusCode::BAD_REQUEST)
                .await;
        let client = Client::new();

        let url = format!("{}{path}", server.url());
        let response = client.post(url).send().await.unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        mock.assert();
    }

    #[tokio::test]
    async fn test_ok() {
        let mut server = start_mock_tmdb_api().await;
        let (mock, path) = mock_get_ok(&mut server).await;

        let client = Client::new();

        let url = format!("{}{path}", server.url());
        let response = client.get(url).send().await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        mock.assert();
    }
}
