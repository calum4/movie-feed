use axum::http::header::CONTENT_TYPE;
use axum::http::{HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use rss::Channel;

pub(super) struct Rss {
    channel: Channel,
}

impl Rss {
    pub(super) fn new(channel: Channel) -> Self {
        Self { channel }
    }
}

impl IntoResponse for Rss {
    fn into_response(self) -> Response {
        match self.channel.pretty_write_to(Vec::new(), b' ', 2) {
            Ok(bytes) => (
                [(
                    CONTENT_TYPE,
                    HeaderValue::from_static("application/rss+xml"),
                )],
                bytes,
            )
                .into_response(),
            Err(error) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                [(
                    CONTENT_TYPE,
                    HeaderValue::from_static(mime::TEXT_PLAIN_UTF_8.as_ref()),
                )],
                error.to_string(),
            )
                .into_response(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_type() {
        let rss = Rss::new(Channel::default());
        let response = rss.into_response();

        assert_eq!(
            response.headers().get(CONTENT_TYPE),
            Some(HeaderValue::from_static("application/rss+xml")).as_ref()
        );
    }
}
