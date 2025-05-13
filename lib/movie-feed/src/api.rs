use crate::api::routes::routes;
use crate::config::Config;
use axum::extract::{FromRequestParts, Request};
use axum::http::HeaderName;
use axum::middleware::Next;
use axum::{middleware, serve};
use axum_client_ip::ClientIp;
use std::net::SocketAddr;
use std::time::Duration;
use thiserror::Error;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use tower_http::request_id::{
    MakeRequestUuid, PropagateRequestIdLayer, RequestId, SetRequestIdLayer,
};
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;
use tracing::{Span, debug_span, info};

mod routes;

const REQUEST_ID_HEADER: HeaderName = HeaderName::from_static("x-request-id");

#[derive(Error, Debug)]
pub(crate) enum ApiError {
    #[error(transparent)]
    BindError(#[from] std::io::Error),
}

pub(crate) async fn start_api_server(config: &Config) -> Result<JoinHandle<()>, ApiError> {
    let addr = SocketAddr::from((config.api.listen_address, config.api.listen_port));

    let listener = TcpListener::bind(addr).await?;

    let router = routes()
        .layer(middleware::from_fn(async |request: Request, next: Next| {
            let span = Span::current();

            let request_id = request
                .extensions()
                .get::<RequestId>()
                .and_then(|id| id.header_value().to_str().ok());

            if let Some(id) = request_id {
                span.record("id", id);
            }

            let (mut parts, body) = request.into_parts();

            if let Ok(ip) = ClientIp::from_request_parts(&mut parts, &()).await {
                span.record("ip", ip.0.to_string());
            }

            next.run(Request::from_parts(parts, body)).await
        }))
        .layer(
            TraceLayer::new_for_http().make_span_with(|request: &Request| {
                debug_span!(
                    "request",
                    id = tracing::field::Empty,
                    method = display(request.method()),
                    path = display(request.uri().path()),
                    ip = tracing::field::Empty,
                )
            }),
        )
        .layer(config.api.client_ip_source.clone().into_extension())
        .layer(PropagateRequestIdLayer::new(REQUEST_ID_HEADER))
        .layer(SetRequestIdLayer::new(REQUEST_ID_HEADER, MakeRequestUuid))
        .layer(TimeoutLayer::new(Duration::from_secs(30)));

    // TODO - Cache responses

    let handle = tokio::spawn(async move {
        serve(
            listener,
            router.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .expect("per axum docs, an error is never returned")
    });

    info!("Listening for API requests");

    Ok(handle)
}
