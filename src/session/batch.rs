use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use urlencoding::encode;

use crate::{common, cryptography, error, invoice, session};

#[derive(Debug)]
pub struct BatchPartSendingInfo {
    pub data: Vec<u8>,
    pub metadata: cryptography::FileMetadata,
    pub ordinal_number: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenBatchSessionRequest {
    #[serde(rename = "formCode")]
    pub form_code: invoice::FormCode,

    #[serde(rename = "batchFile")]
    pub batch_file: BatchFileInfo,

    #[serde(rename = "encryption")]
    pub encryption: cryptography::EncryptionInfo,

    #[serde(rename = "offlineMode")]
    pub offline_mode: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BatchFileInfo {
    #[serde(rename = "fileSize")]
    pub file_size: usize,

    #[serde(rename = "fileHash")]
    pub file_hash: String,

    #[serde(rename = "fileParts")]
    pub file_parts: Vec<BatchFilePartInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BatchFilePartInfo {
    #[serde(rename = "ordinalNumber")]
    pub ordinal_number: usize,

    #[serde(rename = "fileSize")]
    pub file_size: usize,

    #[serde(rename = "fileHash")]
    pub file_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenBatchSessionResponse {
    #[serde(rename = "referenceNumber")]
    pub reference_number: String,

    #[serde(rename = "partUploadRequests")]
    pub part_upload_requests: Vec<PackagePartSignatureInitResponseType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackagePartSignatureInitResponseType {
    #[serde(rename = "method")]
    pub method: String,

    #[serde(rename = "ordinalNumber")]
    pub ordinal_number: usize,

    #[serde(rename = "url")]
    pub url: String,

    #[serde(rename = "headers")]
    pub headers: HashMap<String, String>,
}

pub async fn open_batch_session(
    base_url: &common::Url,
    request: &OpenBatchSessionRequest,
    access_token: &String,
) -> Result<OpenBatchSessionResponse, error::ErrorResponse> {
    let url = "/v2/sessions/batch";

    let reqwest_client = reqwest::Client::new();
    let response = reqwest_client
        .post(base_url.join(url))
        .json(&request)
        .bearer_auth(&access_token)
        .send()
        .await;

    common::response::handle_response::<OpenBatchSessionResponse, error::ErrorResponse>(response)
        .await
}

async fn send_package_parts(
    parts: &Vec<PackagePartSignatureInitResponseType>,
    batch_part_sending_infos: &Vec<BatchPartSendingInfo>,
) -> Result<(), error::ErrorResponse> {
    let mut errors: Vec<String> = Vec::new();

    for part in parts {
        let file_info = match batch_part_sending_infos
            .iter()
            .find(|x| x.ordinal_number == part.ordinal_number)
        {
            Some(file_info) => file_info,
            None => {
                errors.push(format!(
                    "Brak danych dla części paczki {}",
                    part.ordinal_number
                ));
                continue;
            }
        };

        let method = if let Ok(method) = reqwest::Method::from_bytes(part.method.as_bytes()) {
            method
        } else {
            errors.push(format!(
                "Brak metody HTTP dla części paczki {}",
                part.ordinal_number
            ));
            continue;
        };

        let reqwest_client = reqwest::Client::new();
        let mut request = reqwest_client.request(method, &part.url);

        for (k, v) in &part.headers {
            request = request.header(k.as_str(), v.as_str());
        }

        let response = match request.body(file_info.data.clone()).send().await {
            Ok(response) => response,
            Err(e) => {
                errors.push(format!(
                    "Błąd wysyłki części paczki {}: {}",
                    part.ordinal_number, e
                ));
                continue;
            }
        };

        if !response.status().is_success() {
            errors.push(format!(
                "Błąd wysyłki części paczki {}: {}",
                part.ordinal_number,
                response.status()
            ));
            continue;
        }
    }

    if !errors.is_empty() {
        return Err(error::ErrorResponse {
            code: "send_package_parts_error".into(),
            message: errors.join("\n"),
        });
    }

    Ok(())
}

async fn close_batch_session(
    base_url: &common::Url,
    reference_number: &String,
    access_token: &String,
) -> Result<bool, error::ErrorResponse> {
    let url = format!("/v2/sessions/batch/{}/close", encode(reference_number));

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
        return Err(error::ErrorResponse {
            code: status.as_str().to_string(),
            message: format!("Server returned HTTP {}", status),
        });
    }

    Ok(true)
}

async fn get_batch_session_status(
    base_url: &common::Url,
    reference_number: &String,
    access_token: &String,
    max_attempts: i32,
    sleep_time: u64,
) -> Result<session::status::SessionStatusResponse, error::ErrorResponse> {
    let session_status = match common::pooling::pool(
        || {
            session::status::get_session_status(
                base_url,
                &reference_number,
                &access_token,
                max_attempts,
                sleep_time,
            )
        },
        |result| result.status.code == 200,
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

pub async fn send_invoice_batch(
    base_url: &common::Url,
    encryption: cryptography::EncryptionData,
    access_token: &String,
    system_code: &invoice::SystemCode,
    list: &Vec<(String, String)>,
    part_count: usize,
    max_attempts: i32,
    sleep_time: u64,
) -> Result<session::status::SessionStatusResponse, error::ErrorResponse> {
    let form_code = invoice::FormCode {
        system_code: system_code.system_code().into(),
        schema_version: system_code.schema_version().into(),
        value: system_code.value().into(),
    };

    let (zip_bytes, zip_meta) = common::zip::build_zip(&list);

    let encrypted_parts = encrypt_and_split(&zip_bytes, &encryption, Some(part_count));

    if encrypted_parts.len() < 1 {
        return Err(error::ErrorResponse {
            code: "part_size_error".into(),
            message: "the number of parts is less than 1".into(),
        });
    }

    let parts: Vec<_> = encrypted_parts
        .iter()
        .map(|p| BatchFilePartInfo {
            ordinal_number: p.ordinal_number,
            file_size: p.metadata.file_size,
            file_hash: p.metadata.hash_sha.clone(),
        })
        .collect();

    let open_batch_request = OpenBatchSessionRequest {
        form_code: form_code,
        batch_file: BatchFileInfo {
            file_size: zip_meta.file_size,
            file_hash: zip_meta.hash_sha,
            file_parts: parts,
        },
        encryption: encryption.encryption_info,
        offline_mode: false,
    };

    let open_batch_session_response =
        match open_batch_session(&base_url, &open_batch_request, access_token).await {
            Ok(open_batch_session_response) => open_batch_session_response,
            Err(e) => {
                return Err(error::ErrorResponse {
                    code: "open_batch_session".into(),
                    message: e.message,
                });
            }
        };

    if let Err(e) = send_package_parts(
        &open_batch_session_response.part_upload_requests,
        &encrypted_parts,
    )
    .await
    {
        return Err(error::ErrorResponse {
            code: "open_batch_session".into(),
            message: e.message,
        });
    };

    // close session
    let _ = common::pooling::pool(
        || {
            close_batch_session(
                &base_url,
                &open_batch_session_response.reference_number,
                &access_token,
            )
        },
        |result| *result,
        max_attempts,
        sleep_time,
    )
    .await;

    let session_status = match get_batch_session_status(
        &base_url,
        &open_batch_session_response.reference_number,
        &access_token,
        max_attempts,
        sleep_time,
    )
    .await
    {
        Ok(online_session_status) => online_session_status,
        Err(e) => return Err(e),
    };

    Ok(session_status)
}

pub fn encrypt_and_split(
    zip_bytes: &[u8],
    encryption: &cryptography::EncryptionData,
    part_count: Option<usize>,
) -> Vec<BatchPartSendingInfo> {
    let actual_part_count = part_count
        .unwrap_or_else(|| common::zip::calculate_batch_part_quantity(zip_bytes.len() as u64));

    let raw_parts = if actual_part_count <= 1 {
        vec![zip_bytes.to_vec()]
    } else {
        common::zip::split_bytes(zip_bytes, actual_part_count)
    };

    let mut result = Vec::new();

    for (i, part) in raw_parts.into_iter().enumerate() {
        let encrypted = cryptography::encrypt_bytes_with_aes256(
            &part,
            &encryption.cipher_key,
            &encryption.cipher_iv,
        );

        let metadata = cryptography::get_metadata(&encrypted);

        result.push(BatchPartSendingInfo {
            data: encrypted,
            metadata,
            ordinal_number: i + 1,
        });
    }

    result
}
