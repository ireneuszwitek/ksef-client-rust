use base64;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

use urlencoding::encode;

use crate::{common, cryptography, error, invoice, session};

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenOnlineSessionResponse {
    #[serde(rename = "referenceNumber")]
    pub reference_number: String,

    #[serde(rename = "validUntil")]
    pub valid_until: DateTime<FixedOffset>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenOnlineSessionRequest {
    #[serde(rename = "formCode")]
    pub form_code: invoice::FormCode,

    #[serde(rename = "encryption")]
    pub encryption: cryptography::EncryptionInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SendInvoiceRequest {
    #[serde(rename = "invoiceHash")]
    pub invoice_hash: String,

    #[serde(rename = "invoiceSize")]
    pub invoice_size: i64,

    #[serde(rename = "encryptedInvoiceHash")]
    pub encrypted_invoice_hash: String,

    #[serde(rename = "encryptedInvoiceSize")]
    pub encrypted_invoice_size: i64,

    #[serde(rename = "encryptedInvoiceContent")]
    pub encrypted_invoice_content: String,

    #[serde(rename = "offlineMode")]
    pub offline_mode: bool,

    #[serde(rename = "hashOfCorrectedInvoice")]
    pub hash_of_corrected_invoice: Option<String>,
}

pub async fn open_online_session(
    base_url: &common::Url,
    encryption: &cryptography::EncryptionData,
    access_token: &String,
    system_code: &invoice::SystemCode,
) -> Result<OpenOnlineSessionResponse, error::ErrorResponse> {
    let form_code = invoice::FormCode {
        system_code: system_code.system_code().into(),
        schema_version: system_code.schema_version().into(),
        value: system_code.value().into(),
    };

    let request = OpenOnlineSessionRequest {
        form_code,
        encryption: encryption.encryption_info.clone(),
    };

    let url = "/v2/sessions/online";

    let reqwest_client = reqwest::Client::new();
    let response = reqwest_client
        .post(base_url.join(url))
        .json(&request)
        .bearer_auth(&access_token)
        .send()
        .await;

    common::response::handle_response::<OpenOnlineSessionResponse, error::ErrorResponse>(response)
        .await
}

pub async fn close_online_session(
    base_url: &common::Url,
    reference_number: &String,
    access_token: &String,
) -> Result<(), error::ErrorResponse> {
    let url = format!("/v2/sessions/online/{}/close", encode(reference_number));

    let reqwest_client = reqwest::Client::new();
    let response = reqwest_client
        .post(base_url.join(url.as_str()))
        .bearer_auth(&access_token)
        .send()
        .await
        .map_err(|_| error::ErrorResponse {
            code: "request_error".into(),
            message: "Request error".into(),
        })?;

    let status = response.status();

    if !status.is_success() {
        let err = response
            .json::<error::ErrorResponse>()
            .await
            .unwrap_or_else(|_| error::ErrorResponse {
                code: status.as_str().to_string(),
                message: format!("Server returned HTTP {}", status),
            });

        return Err(err);
    }

    Ok(())
}

pub async fn send_invoice(
    base_url: &common::Url,
    reference_number: &String,
    access_token: &String,
    encryption: &cryptography::EncryptionData,
    xml: &String,
) -> Result<common::OperationResponse, error::ErrorResponse> {
    let invoice = xml.as_bytes().to_vec();

    let encrypted_invoice = cryptography::encrypt_bytes_with_aes256(
        &invoice,
        &encryption.cipher_key,
        &encryption.cipher_iv,
    );

    let invoice_metadata = cryptography::get_metadata(&invoice);
    let encrypted_metadata = cryptography::get_metadata(&encrypted_invoice);

    let request = SendInvoiceRequest {
        invoice_hash: invoice_metadata.hash_sha,
        invoice_size: invoice_metadata.file_size as i64,
        encrypted_invoice_hash: encrypted_metadata.hash_sha,
        encrypted_invoice_size: encrypted_metadata.file_size as i64,
        encrypted_invoice_content: STANDARD.encode(encrypted_invoice),
        offline_mode: false,
        hash_of_corrected_invoice: None,
    };

    let url = format!("/v2/sessions/online/{}/invoices", encode(reference_number));

    let reqwest_client = reqwest::Client::new();
    let response = reqwest_client
        .post(base_url.join(url.as_str()))
        .json(&request)
        .bearer_auth(&access_token)
        .send()
        .await;

    common::response::handle_response::<common::OperationResponse, error::ErrorResponse>(response)
        .await
}

pub async fn get_session_status(
    base_url: &common::Url,
    reference_number: &String,
    access_token: &String,
    max_attempts: i32,
    sleep_time: u64,
) -> Result<session::status::SessionStatusResponse, error::ErrorResponse> {
    let session_status = match common::pooling::pool(
        || {
            session::status::get_session_status(
                &base_url,
                &reference_number,
                &access_token,
                max_attempts,
                sleep_time,
            )
        },
        |result| result.invoice_count == result.successful_invoice_count,
        max_attempts,
        sleep_time,
    )
    .await
    {
        Ok(session_status) => session_status,
        Err(_) => {
            return Err(error::ErrorResponse {
                code: "get_session_status_error".into(),
                message: "get_session_status_error".into(), //e.into(),
            });
        }
    };

    Ok(session_status)
}
