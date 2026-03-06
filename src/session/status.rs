use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use urlencoding::encode;

use crate::{common, error};

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionStatusResponse {
    #[serde(rename = "status")]
    pub status: common::OperationStatusInfo,

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

async fn try_get_session_status(
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

pub(crate) async fn get_session_status(
    base_url: &common::Url,
    reference_number: &String,
    access_token: &String,
    max_attempts: i32,
    sleep_time: u64,
) -> Result<SessionStatusResponse, error::ErrorResponse> {
    let mut attempt = 0;

    loop {
        if let Ok(status_response) =
            try_get_session_status(base_url, &reference_number, &access_token).await
        {
            if status_response.successful_invoice_count.is_some() {
                return Ok(status_response);
            }

            if attempt >= max_attempts {
                return Ok(status_response);
            }
        }

        attempt += 1;
        tokio::time::sleep(tokio::time::Duration::from_millis(sleep_time)).await;
    }
}
