pub mod consts;
pub mod error;

pub use crate::consts::MULTI_SERVICE_DEFAULT_API_URL;
pub use crate::error::Error;

pub use crate::error::Result;
use ic_agent::Identity;
use multi_service_types::yral_identity::ic_agent::sign_message;
use multi_service_types::{
    error::MetadataApiError, ApiResult, DeviceRegistrationToken, RegisterDeviceReq,
    RegisterDeviceRes, UnregisterDeviceReq, UnregisterDeviceRes,
};
use reqwest::Url;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct MultiServiceClient<const AUTH: bool> {
    base_url: Url,
    client: reqwest::Client,
    jwt_token: Option<String>,
}

impl Default for MultiServiceClient<false> {
    fn default() -> Self {
        Self {
            base_url: Url::parse(MULTI_SERVICE_DEFAULT_API_URL).unwrap(),
            client: Default::default(),
            jwt_token: None,
        }
    }
}

impl<const A: bool> MultiServiceClient<A> {
    pub fn with_base_url(base_url: Url) -> Self {
        Self {
            base_url,
            client: Default::default(),
            jwt_token: None,
        }
    }

    pub async fn register_device(
        &self,
        identity: &impl Identity,
        registration_token: DeviceRegistrationToken,
    ) -> Result<RegisterDeviceRes> {
        let signature = sign_message(
            identity,
            registration_token
                .clone()
                .try_into()
                .map_err(|_| Error::Api(MetadataApiError::AuthTokenMissing))?,
        )?;
        let sender = identity
            .sender()
            .map_err(|e| Error::Api(MetadataApiError::Unknown(e.to_string())))?;

        let api_url = self
            .base_url
            .join("api/v1/notifications/")
            .map_err(|e| Error::Api(MetadataApiError::Unknown(e.to_string())))?
            .join(&sender.to_text())
            .map_err(|e| Error::Api(MetadataApiError::Unknown(e.to_string())))?;

        let res = self
            .client
            .post(api_url)
            .json(&RegisterDeviceReq {
                registration_token,
                signature,
            })
            .send()
            .await?;

        let res: ApiResult<RegisterDeviceRes> = res.json().await?;
        Ok(res?)
    }

    pub async fn unregister_device(
        &self,
        identity: &impl Identity,
        registration_token: DeviceRegistrationToken,
    ) -> Result<UnregisterDeviceRes> {
        let signature = sign_message(
            identity,
            registration_token
                .clone()
                .try_into()
                .map_err(|_| Error::Api(MetadataApiError::AuthTokenMissing))?,
        )?;
        let sender = identity.sender().map_err(|_| {
            Error::Identity(multi_service_types::yral_identity::error::Error::SenderNotFound)
        })?;
        let api_url = self
            .base_url
            .join("api/v1/notifications/")
            .map_err(|e| Error::Api(MetadataApiError::Unknown(e.to_string())))?
            .join(&sender.to_text())
            .map_err(|e| Error::Api(MetadataApiError::Unknown(e.to_string())))?;

        let res = self
            .client
            .delete(api_url)
            .json(&UnregisterDeviceReq {
                registration_token,
                signature,
            })
            .send()
            .await?;

        let res: ApiResult<UnregisterDeviceRes> = res.json().await?;
        Ok(res?)
    }
}

impl MultiServiceClient<true> {
    pub fn with_jwt_token(self, jwt_token: String) -> Self {
        Self {
            jwt_token: Some(jwt_token),
            ..self
        }
    }
}
