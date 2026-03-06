use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{Cursor, Write};

use urlencoding::encode;

use chrono::{DateTime, Utc};

use crate::{common, cryptography, error, invoice};

const METADATA_ENTRY_NAME: &str = "_metadata.json";
const XML_FILE_EXTENSION: &str = ".xml";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExportInvoiceRequest {
    #[serde(rename = "Encryption")]
    pub encryption: cryptography::EncryptionInfo,

    #[serde(rename = "Filters")]
    pub filters: invoice::InvoiceQueryFilters,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct ExportInvoiceStatusResponse {
    #[serde(rename = "status")]
    pub(crate) status: common::OperationStatusInfo,

    #[serde(rename = "completedDate")]
    pub(crate) completed_date: Option<DateTime<Utc>>,

    #[serde(rename = "packageExpirationDate")]
    pub(crate) package_expiration_date: Option<DateTime<Utc>>,

    #[serde(rename = "package")]
    pub(crate) package: ExportInvoicePackage,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportInvoicePackage {
    #[serde(rename = "invoiceCount")]
    pub invoice_count: i32,

    #[serde(rename = "size")]
    pub size: i64,

    #[serde(rename = "parts")]
    pub parts: Vec<ExportInvoicePackagePart>,

    #[serde(rename = "isTruncated")]
    pub is_truncated: bool,

    #[serde(rename = "lastIssueDate")]
    pub last_issue_date: Option<DateTime<Utc>>,

    #[serde(rename = "lastInvoicingDate")]
    pub last_invoicing_date: Option<DateTime<Utc>>,

    #[serde(rename = "lastPermanentStorageDate")]
    pub last_permanent_storage_date: Option<DateTime<Utc>>,

    #[serde(rename = "permanentStorageHwmDate")]
    pub permanent_storage_hwm_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportInvoicePackagePart {
    #[serde(rename = "ordinalNumber")]
    pub ordinal_number: i32,

    #[serde(rename = "partName")]
    pub part_name: String,

    #[serde(rename = "method")]
    pub method: String,

    #[serde(rename = "url")]
    pub url: String,

    #[serde(rename = "partSize")]
    pub part_size: i64,

    #[serde(rename = "partHash")]
    pub part_hash: String,

    #[serde(rename = "encryptedPartSize")]
    pub encrypted_part_size: i64,

    #[serde(rename = "encryptedPartHash")]
    pub encrypted_part_hash: String,

    #[serde(rename = "expirationDate")]
    pub expiration_date: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InvoicePackageMetadata {
    pub invoices: Option<Vec<invoice::InvoiceSummary>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportInvoiceResult {
    pub metadata_summaries: Vec<invoice::InvoiceSummary>,
    pub xml_files: HashMap<String, String>,
    pub is_truncated: bool,
    pub last_permanent_storage_date: Option<DateTime<Utc>>,
    pub permanent_storage_hwm_date: Option<DateTime<Utc>>,
}

async fn start_export_invoices(
    base_url: &common::Url,
    request: &ExportInvoiceRequest,
    access_token: &String,
) -> Result<common::OperationResponse, error::ErrorResponse> {
    let url = "/v2/invoices/exports";

    let reqwest_client = reqwest::Client::new();
    let response = reqwest_client
        .post(base_url.join(url))
        .json(&request)
        .bearer_auth(&access_token)
        .send()
        .await;

    common::response::handle_response::<common::OperationResponse, error::ApiErrorResponse>(
        response,
    )
    .await
}

async fn get_export_invoice_status(
    base_url: &common::Url,
    reference_number: &String,
    access_token: &String,
) -> Result<ExportInvoiceStatusResponse, error::ErrorResponse> {
    let url = format!("/v2/invoices/exports/{}", encode(reference_number));

    let reqwest_client = reqwest::Client::new();
    let response = reqwest_client
        .get(base_url.join(url.as_str()))
        .bearer_auth(&access_token)
        .send()
        .await;

    common::response::handle_response::<ExportInvoiceStatusResponse, error::ErrorResponse>(response)
        .await
}

pub async fn export_invoice(
    base_url: &common::Url,
    encryption: &cryptography::EncryptionData,
    filters: &invoice::InvoiceQueryFilters,
    access_token: &String,
    max_attempts: i32,
    sleep_time: u64,
) -> Result<ExportInvoiceResult, error::ErrorResponse> {
    let export_invoice_request = ExportInvoiceRequest {
        encryption: encryption.encryption_info.clone(),
        filters: (*filters).clone(),
    };

    let start_export_invoices =
        match start_export_invoices(&base_url, &export_invoice_request, &access_token).await {
            Ok(start_export_invoices) => start_export_invoices,
            Err(e) => {
                return Err(e);
            }
        };

    let export_invoice_status = match common::pooling::pool(
        || {
            get_export_invoice_status(
                &base_url,
                &start_export_invoices.reference_number,
                &access_token,
            )
        },
        |result| result.status.code == 200,
        max_attempts,
        sleep_time,
    )
    .await
    {
        Ok(export_status) => export_status,
        Err(e) => {
            return Err(error::ErrorResponse {
                code: "export_invoice_status_error".into(),
                message: e.into(),
            });
        }
    };

    let mut metadata_summaries: Vec<invoice::InvoiceSummary> = Vec::new();
    let mut xml_files: HashMap<String, String> = HashMap::new();

    if !export_invoice_status.package.parts.is_empty() {
        let decrypted_archive_stream =
            match download_package_parts(&export_invoice_status.package.parts, &encryption).await {
                Ok(decrypted_archive_stream) => decrypted_archive_stream,
                Err(e) => {
                    return Err(error::ErrorResponse {
                        code: "download_package_parts_error".into(),
                        message: e.into(),
                    });
                }
            };

        let unzipped_files = common::zip::unzip(decrypted_archive_stream);

        for (file_name, content) in unzipped_files {
            if file_name.eq_ignore_ascii_case(METADATA_ENTRY_NAME) {
                if let Ok(metadata) = serde_json::from_str::<InvoicePackageMetadata>(&content) {
                    if let Some(invoices) = metadata.invoices {
                        metadata_summaries.extend(invoices);
                    }
                }
            } else if file_name.to_lowercase().ends_with(XML_FILE_EXTENSION) {
                xml_files.insert(file_name.to_lowercase(), content);
            }
        }
    }

    let result = ExportInvoiceResult {
        metadata_summaries: metadata_summaries,
        xml_files: xml_files,
        is_truncated: export_invoice_status.package.is_truncated,
        last_permanent_storage_date: export_invoice_status.package.last_permanent_storage_date,
        permanent_storage_hwm_date: export_invoice_status.package.permanent_storage_hwm_date,
    };

    Ok(result)
}

async fn download_package_parts(
    parts: &Vec<ExportInvoicePackagePart>,
    encryption: &cryptography::EncryptionData,
) -> Result<Cursor<Vec<u8>>, &'static str> {
    let mut buffer = Cursor::new(Vec::new());

    let mut parts_sorted: Vec<_> = parts.iter().collect();
    parts_sorted.sort_by_key(|p| p.ordinal_number);

    for part in parts_sorted {
        let encrypted_bytes = match download_package_part(&part).await {
            Ok(encrypted_bytes) => encrypted_bytes,
            Err(e) => return Err(e),
        };

        let decrypted_bytes = match cryptography::decrypt_bytes_with_aes256(
            &encrypted_bytes,
            &encryption.cipher_key,
            &encryption.cipher_iv,
        ) {
            Ok(decrypted_bytes) => decrypted_bytes,
            Err(_) => return Err("decrypted_bytes_error"),
        };

        buffer.write_all(&decrypted_bytes).unwrap();
    }

    buffer.set_position(0);
    Ok(buffer)
}

async fn download_package_part(part: &ExportInvoicePackagePart) -> Result<Vec<u8>, &'static str> {
    let method_str = if part.method.is_empty() {
        "GET"
    } else {
        part.method.as_str()
    };

    let method = method_str
        .parse::<reqwest::Method>()
        .map_err(|e| format!("Invalid HTTP method: {}", e))
        .unwrap();

    let reqwest_client = reqwest::Client::new();
    let request = reqwest_client.request(method, &part.url);

    let response = request
        .send()
        .await
        .map_err(|e| format!("Response error: {}", e))
        .unwrap();
    let response = response
        .error_for_status()
        .map_err(|e| format!("EnsureSuccessStatusCode error: {}", e))
        .unwrap();

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Get bytes error: {}", e))
        .unwrap();
    Ok(bytes.to_vec())
}
