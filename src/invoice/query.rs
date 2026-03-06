use chrono::{DateTime, offset::Utc};
use serde::{Deserialize, Serialize};

use crate::{common, error, invoice};

#[derive(Debug, Serialize, Deserialize)]
pub struct PagedInvoiceResponse {
    #[serde(rename = "hasMore")]
    pub has_more: bool,

    #[serde(rename = "isTruncated")]
    pub is_truncated: bool,

    #[serde(rename = "invoices")]
    pub invoices: Vec<invoice::InvoiceSummary>,

    #[serde(rename = "permanentStorageHwmDate")]
    pub permanent_storage_hwm_date: Option<DateTime<Utc>>,
}

pub async fn query_metadata(
    base_url: &common::Url,
    request: &invoice::InvoiceQueryFilters,
    access_token: &String,
    page_offset: i32,
    page_size: i32,
    sort_order: invoice::SortOrder,
) -> Result<PagedInvoiceResponse, error::ErrorResponse> {
    let mut url = format!("/v2/invoices/query/metadata?sortOrder={}", sort_order);

    if page_offset > 0 {
        url = format!("{}&pageOffset={}", url, page_offset);
    }

    if page_size > 0 {
        url = format!("{}&pageSize={}", url, page_size);
    }

    let reqwest_client = reqwest::Client::new();
    let response = reqwest_client
        .post(base_url.join(url.as_str()))
        .bearer_auth(access_token)
        .header("Content-Type", "application/json")
        .json(request)
        .send()
        .await;

    common::response::handle_response::<PagedInvoiceResponse, error::ApiErrorResponse>(response)
        .await
}
