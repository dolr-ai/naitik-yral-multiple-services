use serde::{Deserialize, Serialize};
use thiserror::Error;
use utoipa::ToSchema;

#[derive(Deserialize, Serialize, Error, Debug, PartialEq, ToSchema)]
#[non_exhaustive]
pub enum ApiError {
    #[error("invalid signature provided")]
    InvalidSignature,
    #[error("internal error: redis")]
    Redis,
    #[error("internal error: deser")]
    Deser,
    #[error("jwt error - invalid token")]
    Jwt,
    #[error("invalid authentication token")]
    AuthToken,
    #[error("missing authentication token")]
    AuthTokenMissing,
    #[error("failed to delete keys (redis)")]
    DeleteKeys,
    #[error("metadata for principal not found")]
    MetadataNotFound,
    #[error("device not found")]
    DeviceNotFound,
    #[error("notification key not found")]
    NotificationKeyNotFound,
    #[error("unknown: {0}")]
    Unknown(String),
    #[error("invalid email: {0}")]
    InvalidEmail(String),
    #[error("device already registered")]
    DeviceAlreadyRegistered,
    #[error("unauthorized")]
    Unauthorized,
    #[error("environment variable not found")]
    EnvironmentVariable,
    #[error("environment variable missing")]
    EnvironmentVariableMissing,
    #[error("invalid principal")]
    InvalidPrincipal,
    #[error("failed to update session: {0}")]
    UpdateSession(String),
    #[error("invalid username, must be 3-15 alphanumeric characters")]
    InvalidUsername,
    #[error("sqlx error")]
    SqlxError,
    #[error("data parsing error: {0}")]
    DataParseError(String),
    #[error("firebase api error: {0}")]
    FirebaseApiError(String),
}

pub type ApiResult<T> = Result<T, ApiError>;
