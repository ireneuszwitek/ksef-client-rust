use chrono::DateTime;
use chrono::offset::Utc;
use serde::{Deserialize, Serialize};

use crate::{common, error};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct SignatureResponse {
    #[serde(rename = "referenceNumber")]
    pub(crate) reference_number: String,

    #[serde(rename = "authenticationToken")]
    pub(crate) authentication_token: OperationToken,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct OperationToken {
    #[serde(rename = "token")]
    pub(crate) token: String,

    #[serde(rename = "validUntil")]
    pub(crate) valid_until: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct AuthenticationKsefTokenRequest {
    #[serde(rename = "Challenge")]
    pub(crate) challenge: String,
    #[serde(rename = "ContextIdentifier")]
    pub(crate) context_identifier: AuthenticationTokenContextIdentifier,
    #[serde(rename = "EncryptedToken")]
    pub(crate) encrypted_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct AuthenticationTokenContextIdentifier {
    #[serde(rename = "Type")]
    pub(crate) auth_type: AuthenticationTokenContextIdentifierType,
    #[serde(rename = "Value")]
    pub(crate) value: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum AuthenticationTokenContextIdentifierType {
    #[serde(rename = "Nip")]
    Nip,

    #[serde(rename = "InternalId")]
    InternalId,

    #[serde(rename = "NipVatUe")]
    NipVatUe,

    #[serde(rename = "PeppolId")]
    PeppolId,
}

pub(crate) async fn submit_ksef_token_auth_request(
    base_url: &common::Url,
    request: &AuthenticationKsefTokenRequest,
) -> Result<SignatureResponse, error::ErrorResponse> {
    let url = "/v2/auth/ksef-token";

    let reqwest_client = reqwest::Client::new();
    let response = reqwest_client
        .post(base_url.join(url))
        .json(&request)
        .send()
        .await;
    common::response::handle_response::<SignatureResponse, error::ApiErrorResponse>(response).await
}
