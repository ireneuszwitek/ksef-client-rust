use urlencoding::encode;

use chrono::DateTime;
use chrono::offset::Utc;
use serde::{Deserialize, Serialize};

use crate::{common, error};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct AuthStatus {
    #[serde(rename = "startDate")]
    pub(crate) start_date: DateTime<Utc>,

    #[serde(rename = "authenticationMethod")]
    pub(crate) authentication_method: AuthenticationMethodEnum,

    #[serde(rename = "status")]
    pub(crate) status: common::OperationStatusInfo,

    #[serde(rename = "isTokenRedeemed")]
    pub(crate) is_token_redeemed: Option<bool>,

    #[serde(rename = "lastTokenRefreshDate")]
    pub(crate) last_token_refresh_date: Option<DateTime<Utc>>,

    #[serde(rename = "refreshTokenValidUntil")]
    pub(crate) refresh_token_valid_until: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) enum AuthenticationMethodEnum {
    #[serde(rename = "Token")]
    Token,

    #[serde(rename = "TrustedProfile")]
    TrustedProfile,

    #[serde(rename = "InternalCertificate")]
    InternalCertificate,

    #[serde(rename = "QualifiedSignature")]
    QualifiedSignature,

    #[serde(rename = "QualifiedSeal")]
    QualifiedSeal,

    #[serde(rename = "PersonalSignature")]
    PersonalSignature,

    #[serde(rename = "PeppolSignature")]
    PeppolSignature,
}

pub(crate) async fn get_auth_status(
    base_url: &common::Url,
    auth_operation_reference_number: &String,
    authentication_token: &String,
) -> Result<AuthStatus, error::ErrorResponse> {
    let escaped = encode(auth_operation_reference_number);
    let url = format!("/v2/auth/{}", escaped);

    let reqwest_client = reqwest::Client::new();
    let response = reqwest_client
        .get(base_url.join(url.as_str()))
        .bearer_auth(&authentication_token)
        .send()
        .await;

    common::response::handle_response::<AuthStatus, error::ErrorResponse>(response).await
}
