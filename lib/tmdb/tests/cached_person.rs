use http::{Method, StatusCode};
use tmdb::Tmdb;
use tmdb::api_version::ApiVersion;
use tmdb::endpoints::v3::person;
use tmdb_test_utils::api::misc::status_codes::mock_status_code_at_path;
use tmdb_test_utils::api::v3::person::mock_get_person_details;
use tmdb_test_utils::start_mock_tmdb_api;

#[tokio::test]
async fn test_person_cache() {
    const PERSON_ID: i32 = 19498;

    let mut server = start_mock_tmdb_api().await;
    let mock = mock_get_person_details(&mut server, PERSON_ID)
        .await
        .expect(1);

    let mut tmdb = Tmdb::default();
    tmdb.override_api_url(server.url().as_str()).unwrap();

    let _a = person::get(&tmdb, PERSON_ID).await;
    let _b = person::get(&tmdb, PERSON_ID).await;

    mock.assert()
}

#[tokio::test]
async fn test_person_cache_err() {
    const PERSON_ID: i32 = -1;

    let mut server = start_mock_tmdb_api().await;

    let path = format!("/{}person/{PERSON_ID}", ApiVersion::V3.base_path());

    let mock = mock_status_code_at_path(
        &mut server,
        path.as_str(),
        Method::GET,
        StatusCode::NOT_FOUND,
    )
    .await
    .expect(2);

    let mut tmdb = Tmdb::default();
    tmdb.override_api_url(server.url().as_str()).unwrap();

    let _a = person::get(&tmdb, PERSON_ID).await;
    let _b = person::get(&tmdb, PERSON_ID).await;

    mock.assert()
}
