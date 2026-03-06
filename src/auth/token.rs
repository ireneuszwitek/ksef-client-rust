use base64;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;

use chrono::DateTime;
use chrono::offset::Utc;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;

use crate::{
    auth::{auth_status, challenge, ksef_token},
    certificates, common, cryptography, error,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenInfo {
    #[serde(rename = "token")]
    pub token: String,

    #[serde(rename = "validUntil")]
    pub valid_until: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPair {
    #[serde(rename = "accessToken")]
    pub access_token: TokenInfo,

    #[serde(rename = "refreshToken")]
    pub refresh_token: TokenInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RefreshTokenResponse {
    #[serde(rename = "accessToken")]
    pub(crate) access_token: TokenInfo,
}

pub(crate) async fn refresh_access_token(
    base_url: &common::Url,
    refresh_token: &String,
) -> Result<TokenInfo, error::ErrorResponse> {
    let url = "/v2/auth/token/refresh";

    let reqwest_client = reqwest::Client::new();
    let response = reqwest_client
        .post(base_url.join(url))
        .bearer_auth(&refresh_token)
        .send()
        .await;
    match common::response::handle_response::<RefreshTokenResponse, error::ErrorResponse>(response)
        .await
    {
        Ok(refresh_token_response) => Ok(refresh_token_response.access_token),
        Err(e) => Err(e),
    }
}

async fn get_access_token_by_authentication_token(
    base_url: &common::Url,
    authentication_token: &String,
) -> Result<TokenPair, reqwest::Error> {
    let url = "/v2/auth/token/redeem";

    let reqwest_client = reqwest::Client::new();
    let result = reqwest_client
        .post(base_url.join(url))
        .bearer_auth(&authentication_token)
        .send()
        .await?
        .json::<TokenPair>()
        .await?;
    Ok(result)
}

pub(crate) async fn get_access_tokens(
    base_url: &common::Url,
    public_certificates: &RefCell<Option<Vec<certificates::PemCertificateInfo>>>,
    ksef_token: &String,
    nip: &String,
    max_attempts: i32,
    sleep_time: u64,
) -> Result<TokenPair, error::ErrorResponse> {
    let ksef_token_cert = match certificates::public_certificate(
        &base_url,
        &public_certificates,
        &certificates::PublicKeyCertificateUsage::KsefTokenEncryption,
    )
    .await
    {
        Ok(ksef_token_cert) => ksef_token_cert,
        Err(e) => {
            return Err(error::ErrorResponse {
                code: "public_certificate_error".into(),
                message: e.to_string(),
            });
        }
    };

    let challenge = match challenge::get_auth_challenge(&base_url).await {
        Ok(challenge) => challenge,
        Err(e) => {
            return Err(error::ErrorResponse {
                code: "challenge_error".into(),
                message: e.to_string(),
            });
        }
    };

    let timestamp_ms = challenge.timestamp.timestamp_millis();

    let token_with_timestamp = format!("{}|{}", &ksef_token, timestamp_ms);
    let token_bytes: Vec<u8> = token_with_timestamp.as_bytes().to_vec();

    let encrypted: Vec<u8> = match cryptography::encrypt_ksef_token_with_rsa_using_public_key(
        &ksef_token_cert,
        &token_bytes,
    ) {
        Ok(encrypted) => encrypted,
        Err(e) => {
            return Err(error::ErrorResponse {
                code: "encrypt_ksef_token_with_rsa_using_public_key_error".into(),
                message: e.to_string(),
            });
        }
    };

    let encrypted_token_b64 = STANDARD.encode(&encrypted);

    let request = ksef_token::AuthenticationKsefTokenRequest {
        challenge: challenge.challenge,
        context_identifier: ksef_token::AuthenticationTokenContextIdentifier {
            auth_type: ksef_token::AuthenticationTokenContextIdentifierType::Nip,
            value: Some(nip.clone()),
        },
        encrypted_token: encrypted_token_b64,
    };

    let signature = match ksef_token::submit_ksef_token_auth_request(&base_url, &request).await {
        Ok(signature) => signature,
        Err(e) => {
            return Err(e);
        }
    };

    let _ = common::pooling::pool(
        || {
            auth_status::get_auth_status(
                &base_url,
                &signature.reference_number,
                &signature.authentication_token.token,
            )
        },
        |result| result.status.code == 200,
        max_attempts,
        sleep_time,
    )
    .await;

    let tokens = match get_access_token_by_authentication_token(
        &base_url,
        &signature.authentication_token.token,
    )
    .await
    {
        Ok(tokens) => tokens,
        Err(e) => {
            return Err(error::ErrorResponse {
                code: "token_error".into(),
                message: e.to_string(),
            });
        }
    };

    Ok(tokens)
}
