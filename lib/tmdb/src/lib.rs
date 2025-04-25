pub mod endpoints;
pub mod models;

use reqwest::Client;

pub const API_HOST: &str = "api.themoviedb.org";
pub const API_VERSION: u8 = 3;

pub struct Tmdb {
    token: String,
    http_client: Client,
}

impl Tmdb {
    pub fn new(http_client: Client, token: String) -> Self {
        Self { token, http_client }
    }
}

#[cfg(test)]
mod tests {
    use std::any::{Any, TypeId};
    use super::*;

    #[test]
    fn test_new() {
        let tmdb = Tmdb::new(Client::new(), "NO_TOKEN_REQUIRED".to_string());
        
        assert_eq!(tmdb.token, "NO_TOKEN_REQUIRED");
        assert_eq!(tmdb.http_client.type_id(), TypeId::of::<Client>());
    }
}
