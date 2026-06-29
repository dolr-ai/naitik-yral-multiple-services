use crate::metadata_types::error::MetadataApiError;
use crate::yral_identity::Error as IdentityError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("{0}")]
    Api(#[from] MetadataApiError),
    #[error("failed to sign: {0}")]
    Identity(#[from] IdentityError),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
