use http::StatusCode;
use reqwest::Response;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::ops::Deref;
use tracing::warn;

/// Not guaranteed to be a valid TMDB error code
#[derive(Debug, Copy, Clone, Eq, PartialEq, serde::Deserialize)]
pub struct TmdbErrorCode(u8);

impl TmdbErrorCode {
    pub fn new(error: u8) -> Self {
        Self(error)
    }
}

impl Display for TmdbErrorCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl Deref for TmdbErrorCode {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

macro_rules! construct_tmdb_error {
    ($({
        variant: $variant:ident,
        code: $code:literal,
        status: StatusCode::$status:ident,
        message: $message:literal
    }),* $(,)? ) => {
        #[derive(Debug, Copy, Clone, Eq, PartialEq)]
        #[non_exhaustive]
        pub enum TmdbError {
            $($variant,)*
        }

        impl TmdbError {
            pub const fn message(&self) -> &'static str {
                match self {
                    $(Self::$variant => $message,)*
                }
            }

            #[allow(unreachable_patterns)]
            pub const fn hint_valid_status_code(status_code: StatusCode) -> bool {
                match status_code {
                    $(StatusCode::$status => true,)*
                    _ => false,
                }
            }
        }

        impl TryFrom<(StatusCode, TmdbErrorCode)> for TmdbError {
            type Error = UnknownTmdbError;

            fn try_from(value: (StatusCode, TmdbErrorCode)) -> Result<Self, Self::Error> {
                Ok(match value {
                    $((StatusCode::$status, TmdbErrorCode($code)) => Self::$variant,)*
                    (status_code, tmdb_code) => return Err(UnknownTmdbError::new(status_code, Some(tmdb_code))),
                })
            }
        }
    };
}

impl Display for TmdbError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.message())
    }
}

impl Error for TmdbError {}

impl TmdbError {
    pub(crate) async fn try_from_response(response: Response) -> Result<Self, UnknownTmdbError> {
        let status = response.status();

        if !TmdbError::hint_valid_status_code(status) {
            return Err(UnknownTmdbError::UnknownStatus(status));
        }

        #[derive(serde::Deserialize)]
        struct ErrorData {
            #[serde(rename = "status_code")]
            tmdb_error_code: TmdbErrorCode,
        }

        match response.json::<ErrorData>().await {
            Ok(error_data) => TmdbError::try_from((status, error_data.tmdb_error_code)),
            Err(error) => {
                warn!("unable to extract error data: {error}");
                Err(UnknownTmdbError::UnknownStatus(status))
            }
        }
    }
}

construct_tmdb_error!(
    {variant: Success, code: 1, status: StatusCode::OK, message: "Success."},
    {variant: InvalidService, code: 2, status: StatusCode::NOT_IMPLEMENTED, message: "Invalid service: this service does not exist."},
    {variant: InsufficientPermission, code: 3, status: StatusCode::UNAUTHORIZED, message: "Authentication failed: You do not have permissions to access the service."},
    {variant: InvalidFormat, code: 4, status: StatusCode::METHOD_NOT_ALLOWED, message: "Invalid format: This service doesn't exist in that format."},
    {variant: InvalidParameters, code: 5, status: StatusCode::UNPROCESSABLE_ENTITY, message: "Invalid parameters: Your request parameters are incorrect."},
    {variant: InvalidId, code: 6, status: StatusCode::NOT_FOUND, message: "Invalid id: The pre-requisite id is invalid or not found."},
    {variant: InvalidApiKey, code: 7, status: StatusCode::UNAUTHORIZED, message: "Invalid API key: You must be granted a valid key."},
    {variant: DuplicateEntry, code: 8, status: StatusCode::FORBIDDEN, message: "Duplicate entry: The data you tried to submit already exists."},
    {variant: ServiceOffline, code: 9, status: StatusCode::SERVICE_UNAVAILABLE, message: "Service offline: This service is temporarily offline, try again later."},
    {variant: SuspendedApiKey, code: 10, status: StatusCode::UNAUTHORIZED, message: "Suspended API key: Access to your account has been suspended, contact TMDB."},
    {variant: InternalError, code: 11, status: StatusCode::INTERNAL_SERVER_ERROR, message: "Internal error: Something went wrong, contact TMDB."},
    {variant: ItemUpdateSuccess, code: 12, status: StatusCode::CREATED, message: "The item/record was updated successfully."},
    {variant: ItemDeleteSuccess, code: 13, status: StatusCode::OK, message: "The item/record was deleted successfully."},
    {variant: AuthenticationFailed, code: 14, status: StatusCode::UNAUTHORIZED, message: "Authentication failed."},
    {variant: Failed, code: 15, status: StatusCode::INTERNAL_SERVER_ERROR, message: "Failed."},
    {variant: DeviceDenied, code: 16, status: StatusCode::UNAUTHORIZED, message: "Device denied."},
    {variant: SessionDenied, code: 17, status: StatusCode::UNAUTHORIZED, message: "Session denied."},
    {variant: ValidationFailed, code: 18, status: StatusCode::BAD_REQUEST, message: "Validation failed."},
    {variant: InvalidAcceptHeader, code: 19, status: StatusCode::NOT_ACCEPTABLE, message: "Invalid accept header."},
    {variant: InvalidDateRange, code: 20, status: StatusCode::UNPROCESSABLE_ENTITY, message: "Invalid date range: Should be a range no longer than 14 days."},
    {variant: EntryNotFound, code: 21, status: StatusCode::OK, message: "Entry not found: The item you are trying to edit cannot be found."},
    {variant: InvalidPage, code: 22, status: StatusCode::BAD_REQUEST, message: "Invalid page: Pages start at 1 and max at 500. They are expected to be an integer."},
    {variant: InvalidDate, code: 23, status: StatusCode::BAD_REQUEST, message: "Invalid date: Format needs to be YYYY-MM-DD."},
    {variant: TimedOut, code: 24, status: StatusCode::GATEWAY_TIMEOUT, message: "Your request to the backend server timed out. Try again."},
    {variant: RateLimited, code: 25, status: StatusCode::TOO_MANY_REQUESTS, message: "Your request count (#) is over the allowed limit of (40)."},
    {variant: UserAndPassRequired, code: 26, status: StatusCode::BAD_REQUEST, message: "You must provide a username and password."},
    {variant: TooManyResponseObjects, code: 27, status: StatusCode::BAD_REQUEST, message: "Too many append to response objects: The maximum number of remote calls is 20."},
    {variant: InvalidTimezone, code: 28, status: StatusCode::BAD_REQUEST, message: "Invalid timezone: Please consult the documentation for a valid timezone."},
    {variant: RequiresConfirmation, code: 29, status: StatusCode::BAD_REQUEST, message: "You must confirm this action: Please provide a confirm=true parameter."},
    {variant: InvalidUserOrPass, code: 30, status: StatusCode::UNAUTHORIZED, message: "Invalid username and/or password: You did not provide a valid login."},
    {variant: AccountDisabled, code: 31, status: StatusCode::UNAUTHORIZED, message: "Account disabled: Your account is no longer active. Contact TMDB if this is an error."},
    {variant: EmailNotVerified, code: 32, status: StatusCode::UNAUTHORIZED, message: "Email not verified: Your email address has not been verified."},
    {variant: InvalidRequestToken, code: 33, status: StatusCode::UNAUTHORIZED, message: "Invalid request token: The request token is either expired or invalid."},
    {variant: ResourceNotFound, code: 34, status: StatusCode::NOT_FOUND, message: "The resource you requested could not be found."},
    {variant: InvalidToken, code: 35, status: StatusCode::UNAUTHORIZED, message: "Invalid token."},
    {variant: TokenRequireWritePerm, code: 36, status: StatusCode::UNAUTHORIZED, message: "This token hasn't been granted write permission by the user."},
    {variant: InvalidSession, code: 37, status: StatusCode::NOT_FOUND, message: "The requested session could not be found."},
    {variant: RequiresEditPermission, code: 38, status: StatusCode::UNAUTHORIZED, message: "You don't have permission to edit this resource."},
    {variant: Private, code: 39, status: StatusCode::UNAUTHORIZED, message: "This resource is private."},
    {variant: NothingToUpdate, code: 40, status: StatusCode::OK, message: "Nothing to update."},
    {variant: TokenNotApproved, code: 41, status: StatusCode::UNPROCESSABLE_ENTITY, message: "This request token hasn't been approved by the user."},
    {variant: ReqMethodNotSupported, code: 42, status: StatusCode::METHOD_NOT_ALLOWED, message: "This request method is not supported for this resource."},
    {variant: NoBackendConnection, code: 43, status: StatusCode::BAD_GATEWAY, message: "Couldn't connect to the backend server."},
    {variant: OtherInvalidId, code: 44, status: StatusCode::INTERNAL_SERVER_ERROR, message: "The ID is invalid."},
    {variant: UserSuspended, code: 45, status: StatusCode::FORBIDDEN, message: "This user has been suspended."},
    {variant: Maintenance, code: 46, status: StatusCode::SERVICE_UNAVAILABLE, message: "The API is undergoing maintenance. Try again later."},
    {variant: InvalidInput, code: 47, status: StatusCode::BAD_REQUEST, message: "The input is not valid."},
);

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum UnknownTmdbError {
    UnknownStatus(StatusCode),
    UnknownError {
        status_code: StatusCode,
        tmdb_code: TmdbErrorCode,
    },
}

impl UnknownTmdbError {
    fn new(status_code: StatusCode, tmdb_code: Option<TmdbErrorCode>) -> Self {
        match tmdb_code {
            None => Self::UnknownStatus(status_code),
            Some(tmdb_code) => Self::UnknownError {
                status_code,
                tmdb_code,
            },
        }
    }
}

impl Display for UnknownTmdbError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            UnknownTmdbError::UnknownStatus(status) => write!(f, "unknown status code: {status}"),
            UnknownTmdbError::UnknownError {
                status_code,
                tmdb_code,
            } => write!(
                f,
                "unknown tmdb error {{status_code: {status_code}, tmdb_code: {tmdb_code}}}"
            ),
        }
    }
}

impl Error for UnknownTmdbError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_from() {
        assert_eq!(
            TmdbError::try_from((StatusCode::INTERNAL_SERVER_ERROR, TmdbErrorCode(44))),
            Ok(TmdbError::OtherInvalidId)
        );
        assert_eq!(
            TmdbError::try_from((StatusCode::INTERNAL_SERVER_ERROR, TmdbErrorCode(45))),
            Err(UnknownTmdbError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                Some(TmdbErrorCode(45)),
            ))
        );
    }

    #[tokio::test]
    async fn test_from_response() {
        let status_code = StatusCode::BAD_REQUEST;

        let response: Response = http::Response::builder()
            .status(status_code)
            .body(r#"{"status_code": 22}"#)
            .unwrap()
            .into();

        let error = TmdbError::try_from_response(response).await;
        assert_eq!(error, Ok(TmdbError::InvalidPage));
    }

    #[tokio::test]
    async fn test_from_response_invalid_status() {
        let status_code = StatusCode::from_u16(600).unwrap();

        let response: Response = http::Response::builder()
            .status(status_code)
            .body("{}")
            .unwrap()
            .into();

        let error = TmdbError::try_from_response(response).await;
        assert_eq!(error, Err(UnknownTmdbError::UnknownStatus(status_code)));
    }

    #[tokio::test]
    async fn test_from_response_deserialisation_failure() {
        let status_code = StatusCode::OK;

        let response: Response = http::Response::builder()
            .status(status_code)
            .body("{}")
            .unwrap()
            .into();

        let error = TmdbError::try_from_response(response).await;
        assert_eq!(error, Err(UnknownTmdbError::UnknownStatus(status_code)));
    }

    #[test]
    fn test_message() {
        let error = TmdbError::RateLimited;
        assert_eq!(
            error.message(),
            "Your request count (#) is over the allowed limit of (40)."
        );
    }
}
