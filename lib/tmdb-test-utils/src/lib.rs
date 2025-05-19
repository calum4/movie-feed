pub use mockito;
use mockito::{Server, ServerGuard};

pub mod api;

pub async fn start_mock_tmdb_api() -> ServerGuard {
    Server::new_async().await
}
