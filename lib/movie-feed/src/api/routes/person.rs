use axum::Router;

mod combined_credits;

pub(super) const PATH: &str = "/person";

pub(super) fn router() -> Router {
    Router::new().nest(combined_credits::PATH, combined_credits::router())
}
