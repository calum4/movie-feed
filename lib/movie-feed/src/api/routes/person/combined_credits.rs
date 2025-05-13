use axum::Router;
use axum::http::StatusCode;
use axum::routing::get;

pub(super) const PATH: &str = "/{person_id}/combined_credits";

pub(super) fn router() -> Router {
    Router::new().route("/", get(get::combined_credits))
}

mod get {
    use super::*;
    use crate::api::ApiState;
    use axum::Extension;
    use axum::extract::Path;
    use axum::response::{IntoResponse, Response};
    use std::sync::Arc;
    use tracing::warn;

    pub(super) async fn combined_credits(
        Path(person_id): Path<String>,
        api_state: Extension<Arc<ApiState>>,
    ) -> Response {
        let credits =
            tmdb::endpoints::person::combined_credits::get(&api_state.tmdb, &person_id).await;

        match credits {
            Ok(_credits) => {
                todo!()
            }
            Err(error) => {
                warn!("{error}");

                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}
