mod ok;

use axum::Router;

pub(super) fn routes() -> Router {
    Router::new()
        .nest(ok::PATH, ok::router())
}