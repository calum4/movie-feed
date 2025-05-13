use axum::Router;
use axum::http::StatusCode;
use axum::routing::get;

pub(super) const PATH: &str = "/ok";

pub(super) fn router() -> Router {
    Router::new().route("/", get(get::ok))
}

mod get {
    use super::*;

    pub(super) async fn ok() -> StatusCode {
        StatusCode::OK
    }
}
