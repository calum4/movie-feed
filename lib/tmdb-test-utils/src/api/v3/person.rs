pub mod combined_credits;

use crate::api::file_path;
use http::header::CONTENT_TYPE;
use mockito::{Mock, ServerGuard};
use tmdb::api_version::ApiVersion;

pub async fn mock_get_person_details(server: &mut ServerGuard, person_id: i32) -> Mock {
    let api_version = ApiVersion::V3;
    let path = format!("/{}person/{person_id}", api_version.base_path());

    server
        .mock("GET", path.as_str())
        .with_status(200)
        .with_header(CONTENT_TYPE, "application/json")
        .with_body_from_file(file_path(path.as_str(), "GET.json"))
        .create_async()
        .await
}
