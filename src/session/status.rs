use chrono::{DateTime, FixedOffset, Utc};
use serde::{Deserialize, Serialize};
use urlencoding::encode;
use std::fmt;

use crate::{common, error};

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
pub enum SessionType {
    #[serde(rename = "online")]
    Online,
    #[serde(rename = "batch")]
    Batch,
}


impl fmt::Display for SessionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            SessionType::Online => "Online",
            SessionType::Batch => "Batch",
        };
        write!(f, "{}", text)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionStatus {
    #[serde(rename = "succeeded")]
    Succeeded,

    #[serde(rename = "inProgress")]
    InProgress,

    #[serde(rename = "failed")]
    Failed,

    #[serde(rename = "cancelled")]
    Cancelled,
}

impl fmt::Display for SessionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            SessionStatus::Succeeded => "succeeded",
            SessionStatus::InProgress => "inProgress",
            SessionStatus::Failed => "failed",
            SessionStatus::Cancelled => "cancelled",
        };
        write!(f, "{}", text)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SessionsFilter {
    /// Session reference number
    #[serde(rename = "ReferenceNumber")]
    pub reference_number: Option<String>,

    /// Session creation date (from)
    #[serde(rename = "DateCreatedFrom")]
    pub date_created_from: Option<DateTime<Utc>>,

    /// Session creation date (to)
    #[serde(rename = "DateCreatedTo")]
    pub date_created_to: Option<DateTime<Utc>>,

    /// Session closing date (from)
    #[serde(rename = "DateClosedFrom")]
    pub date_closed_from: Option<DateTime<Utc>>,

    /// Session closing date (to)
    #[serde(rename = "DateClosedTo")]
    pub date_closed_to: Option<DateTime<Utc>>,

    /// Date of last activity (from)
    #[serde(rename = "DateModifiedFrom")]
    pub date_modified_from: Option<DateTime<Utc>>,

    /// Date of last activity (to)
    #[serde(rename = "DateModifiedTo")]
    pub date_modified_to: Option<DateTime<Utc>>,

    /// Session statuses
    #[serde(rename = "Statuses")]
    pub statuses: Option<Vec<SessionStatus>>,
}

impl SessionsFilter {
    pub fn get_query_url(&self) -> String {
        let mut out = String::new();

        fn add(out: &mut String, name: &str, value: &str) {
            if !value.is_empty() {
                out.push('&');
                out.push_str(name);
                out.push('=');
                out.push_str(&encode(value));
            }
        }

        // referenceNumber
        if let Some(v) = &self.reference_number {
            add(&mut out, "referenceNumber", v);
        }

        // dates
        if let Some(v) = self.date_created_from {
            add(&mut out, "dateCreatedFrom", &v.to_rfc3339());
        }
        if let Some(v) = self.date_created_to {
            add(&mut out, "dateCreatedTo", &v.to_rfc3339());
        }
        if let Some(v) = self.date_closed_from {
            add(&mut out, "dateClosedFrom", &v.to_rfc3339());
        }
        if let Some(v) = self.date_closed_to {
            add(&mut out, "dateClosedTo", &v.to_rfc3339());
        }
        if let Some(v) = self.date_modified_from {
            add(&mut out, "dateModifiedFrom", &v.to_rfc3339());
        }
        if let Some(v) = self.date_modified_to {
            add(&mut out, "dateModifiedTo", &v.to_rfc3339());
        }
        // statuses
        if let Some(list) = &self.statuses {
            for status in list {
                add(&mut out, "statuses", &status.to_string());
            }
        }

        out
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionsListResponse {
    #[serde(rename = "continuationToken")]
    pub continuation_token: Option<String>,

    #[serde(rename = "sessions")]
    pub sessions: Vec<Session>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    #[serde(rename = "referenceNumber")]
    pub reference_number: String,

    #[serde(rename = "status")]
    pub status: common::OperationStatusInfo,

    #[serde(rename = "dateCreated")]
    pub date_created: DateTime<Utc>,

    #[serde(rename = "dateUpdated")]
    pub date_updated: DateTime<Utc>,

    #[serde(rename = "validUntil")]
    pub valid_until: DateTime<Utc>,

    #[serde(rename = "totalInvoiceCount")]
    pub total_invoice_count: i32,

    #[serde(rename = "successfulInvoiceCount")]
    pub successful_invoice_count: i32,

    #[serde(rename = "failedInvoiceCount")]
    pub failed_invoice_count: i32,
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


pub(crate) async fn get_sessions(
    base_url: &common::Url,
    session_type: SessionType,
    access_token: &String,
    page_size: i32,
    session_filter: &Option<SessionsFilter>,
) -> Result<SessionsListResponse, error::ErrorResponse> {
    let mut url = format!("/v2/sessions?sessionType={}", session_type);

    if page_size > 0 {
        url = format!("{}&pageSize={}", url, page_size);
    }    

    if let Some(filter) = session_filter{
        url = format!("{}{}", url, filter.get_query_url());
    }

    let reqwest_client = reqwest::Client::new();
    let response = reqwest_client
        .get(base_url.join(url.as_str()))
        .bearer_auth(&access_token)
        .send()
        .await;

    common::response::handle_response::<SessionsListResponse, error::ErrorResponse>(response).await
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
