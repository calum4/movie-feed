mod ok;
mod person;

use axum::Router;

pub(super) fn routes() -> Router {
    Router::new()
        .nest(ok::PATH, ok::router())
        .nest(person::PATH, person::router())
}
