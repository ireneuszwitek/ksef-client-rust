use chrono::{DateTime, FixedOffset, Utc};
use serde::{Deserialize, Serialize};
use urlencoding::encode;

use crate::{common, error, invoice};

#[derive(Debug, Serialize, Deserialize)]
pub struct UpoResponse {
    #[serde(rename = "pages")]
    pub pages: Vec<UpoPageResponse>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpoPageResponse {
    #[serde(rename = "referenceNumber")]
    pub reference_number: String,

    #[serde(rename = "downloadUrl")]
    pub download_url: String,

    #[serde(rename = "downloadUrlExpirationDate")]
    pub download_url_expiration_date: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionStatusResponse {
    #[serde(rename = "status")]
    pub status: common::OperationStatusInfo,

    #[serde(rename = "upo")]
    pub upo: Option<UpoResponse>,

    #[serde(rename = "invoiceCount")]
    pub invoice_count: Option<i32>,

    #[serde(rename = "successfulInvoiceCount")]
    pub successful_invoice_count: Option<i32>,

    #[serde(rename = "failedInvoiceCount")]
    pub failed_invoice_count: Option<i32>,

    #[serde(rename = "validUntil")]
    pub valid_until: Option<DateTime<FixedOffset>>,

    #[serde(rename = "dateCreated")]
    pub date_created: DateTime<FixedOffset>,

    #[serde(rename = "dateUpdated")]
    pub date_updated: DateTime<FixedOffset>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionInvoice {
    #[serde(rename = "ordinalNumber")]
    pub ordinal_number: i32,

    #[serde(rename = "invoiceNumber")]
    pub invoice_number: Option<String>,

    #[serde(rename = "ksefNumber")]
    pub ksef_number: Option<String>,

    #[serde(rename = "referenceNumber")]
    pub reference_number: Option<String>,

    #[serde(rename = "invoiceHash")]
    pub invoice_hash: Option<String>,

    #[serde(rename = "invoiceFileName")]
    pub invoice_file_name: Option<String>,

    #[serde(rename = "acquisitionDate")]
    pub acquisition_date: Option<DateTime<Utc>>,

    #[serde(rename = "invoicingDate")]
    pub invoicing_date: DateTime<Utc>,

    #[serde(rename = "permanentStorageDate")]
    pub permanent_storage_date: Option<DateTime<Utc>>,

    #[serde(rename = "upoDownloadUrl")]
    pub upo_download_url: Option<String>,

    // #[serde(rename = "status")]
    pub status: invoice::InvoiceStatusInfo,

    #[serde(rename = "invoicingMode")]
    pub invoicing_mode: Option<invoice::InvoicingMode>,

    #[serde(rename = "upoDownloadUrlExpirationDate")]
    pub upo_download_url_expiration_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionInvoicesResponse {
    #[serde(rename = "continuationToken")]
    pub continuation_token: Option<String>,

    #[serde(rename = "invoices")]
    pub invoices: Vec<SessionInvoice>,
}

pub(crate) async fn get_session_status(
    base_url: &common::Url,
    reference_number: &String,
    access_token: &String,
) -> Result<SessionStatusResponse, error::ErrorResponse> {
    let url = format!("/v2/sessions/{}", encode(reference_number));

    let reqwest_client = reqwest::Client::new();
    let response = reqwest_client
        .get(base_url.join(url.as_str()))
        .bearer_auth(&access_token)
        .send()
        .await;

    common::response::handle_response::<SessionStatusResponse, error::ErrorResponse>(response).await
}

pub(crate) async fn try_get_session_status(
    base_url: &common::Url,
    reference_number: &String,
    access_token: &String,
    max_attempts: i32,
    sleep_time: u64,
) -> Result<SessionStatusResponse, error::ErrorResponse> {
    let mut attempt = 0;

    loop {
        if let Ok(status_response) =
            get_session_status(base_url, &reference_number, &access_token).await
        {
            if status_response.successful_invoice_count.is_some() {
                return Ok(status_response);
            }

            if let Some(failed_invoice_count) = status_response.failed_invoice_count {
                return Err(error::ErrorResponse {
                    code: "failed_invoice".into(),
                    message: format!("failed_invoice_count: {}", failed_invoice_count),
                });
            }

            if attempt >= max_attempts {
                return Ok(status_response);
            }
        }

        attempt += 1;
        tokio::time::sleep(tokio::time::Duration::from_millis(sleep_time)).await;
    }
}

pub(crate) async fn get_session_invoice(
    base_url: &common::Url,
    reference_number: &String,
    access_token: &String,
    page_size: i32,
) -> Result<SessionInvoicesResponse, error::ErrorResponse> {
    let mut url = format!("/v2/sessions/{}/invoices", encode(reference_number));

    if page_size > 0 {
        url = format!("{}?pageSize={}", url, page_size);
    }

    let reqwest_client = reqwest::Client::new();
    let response = reqwest_client
        .get(base_url.join(url.as_str()))
        .bearer_auth(&access_token)
        .send()
        .await;

    common::response::handle_response::<SessionInvoicesResponse, error::ErrorResponse>(response)
        .await
}

pub(crate) async fn get_session_upo(
    base_url: &common::Url,
    session_reference_number: &String,
    upo_reference_number: &String,
    access_token: &String,
) -> Result<String, error::ErrorResponse> {
    let url = format!(
        "/v2/sessions/{}/upo/{}",
        encode(session_reference_number),
        encode(upo_reference_number)
    );

    let reqwest_client = reqwest::Client::new();
    let response = reqwest_client
        .get(base_url.join(url.as_str()))
        .bearer_auth(&access_token)
        .send()
        .await
        .map_err(|_| error::ErrorResponse {
            code: "network_error".into(),
            message: "Failed to send request".into(),
        })?;

    let status = response.status();        

    if !status.is_success() {
                        return Err(error::ErrorResponse {
                    code: status.as_str().to_string(),
                    message: format!("Server returned HTTP {}", status),
                });

    }

    return Ok(response.text().await.unwrap_or_default());
 
}
