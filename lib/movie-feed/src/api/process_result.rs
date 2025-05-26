use axum::response::{IntoResponse, Response};
use tmdb::endpoints::RequestError;
use tmdb::models::v3::tmdb_error::TmdbError;
use tracing::{error, warn};

pub(super) enum ProcessedResponse<T> {
    Ok(T),
    Err(RequestError),
    Response(Response),
}

impl<T> From<RequestError> for ProcessedResponse<T> {
    fn from(error: RequestError) -> Self {
        Self::Err(error)
    }
}

impl<T> From<Response> for ProcessedResponse<T> {
    fn from(response: Response) -> Self {
        ProcessedResponse::Response(response)
    }
}

pub(super) fn process_response<T>(response: Result<T, RequestError>) -> ProcessedResponse<T> {
    match response {
        Ok(response) => ProcessedResponse::Ok(response),
        Err(RequestError::TmdbError(error)) => {
            use TmdbError::*;

            match error {
                Success => (),
                ItemUpdateSuccess => (),
                ItemDeleteSuccess => (),
                InvalidService
                | InsufficientPermission
                | InvalidFormat
                | InvalidApiKey
                | ServiceOffline
                | SuspendedApiKey
                | InternalError
                | AuthenticationFailed
                | DeviceDenied
                | SessionDenied
                | InvalidAcceptHeader
                | UserAndPassRequired
                | InvalidUserOrPass
                | AccountDisabled
                | EmailNotVerified
                | InvalidRequestToken
                | InvalidToken
                | TokenRequireWritePerm
                | InvalidSession
                | RequiresEditPermission
                | Private
                | TokenNotApproved
                | ReqMethodNotSupported
                | NoBackendConnection
                | UserSuspended
                | Maintenance => error!("{error}"),
                _ => warn!("{error}"),
            }

            (
                error.status_code(),
                format!("error received from tmdb: {}", error.message()),
            )
                .into_response()
                .into()
        }
        Err(error) => ProcessedResponse::Err(error),
    }
}
