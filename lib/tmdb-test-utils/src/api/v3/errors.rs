use crate::api::file_path;
use http::header::CONTENT_TYPE;
use mockito::{Mock, ServerGuard};
use tmdb::api_version::ApiVersion;

pub async fn mock_invalid_id(server: &mut ServerGuard) -> (Mock, String) {
    let api_version = ApiVersion::V3;
    let path = format!("/{}errors/invalid_id", api_version.base_path());

    (
        server
            .mock("GET", path.as_str())
            .with_status(404)
            .with_header(CONTENT_TYPE, "application/json")
            .with_body_from_file(file_path(path.as_str(), "GET.json"))
            .create_async()
            .await,
        path,
    )
}
