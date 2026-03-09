use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use urlencoding::encode;

use crate::{common, error, invoice};

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
