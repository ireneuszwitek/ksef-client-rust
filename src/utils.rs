use std::collections::HashMap;
use std::io::{Read, Seek, Write, Cursor};
use tokio::time::{Duration, sleep};
use zip::ZipArchive;
use zip::write::SimpleFileOptions;
use zip::{ZipWriter, CompressionMethod};
use std::cmp::min;

const MAX_PART_SIZE_BYTES: u64 = 100 * 1000 * 1000; // 100 MB


use crate::{cryptography, models};


pub(crate) fn unzip<R: Read + Seek>(zip_stream: R) -> HashMap<String, String> {
    let mut archive = ZipArchive::new(zip_stream).unwrap();
    let mut files = HashMap::new();

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).unwrap();

        if entry.name().trim().is_empty() {
            continue;
        }

        let mut content = String::new();
        entry.read_to_string(&mut content).unwrap();

        files.insert(entry.name().to_string(), content);
    }

    files
}

pub(crate) async fn pool<T, FAction, FutA, FCond>(action: FAction, condition: FCond, max_attempts: i32, delay_ms: u64) -> Result<T, &'static str>
where
    FAction: Fn() -> FutA,
    FutA: Future<Output = Result<T, models::ErrorResponse>>,
    FCond: Fn(&T) -> bool,
{
    for _ in 1..=max_attempts {
        match action().await {
            Ok(result) => {
                if condition(&result) {
                    return Ok(result);
                }
            }
            Err(_) => {
                return Err("get_status_error");
            }
        }

        sleep(Duration::from_millis(delay_ms)).await;
    }

    Err("Maximum number of attempts exceeded")    
}


pub(crate) async fn handle_response<T>(
    response: Result<reqwest::Response, reqwest::Error>,
) -> Result<T, models::ErrorResponse>
where
    T: serde::de::DeserializeOwned,
{
    let response = response.map_err(|_| models::ErrorResponse {
        code: "request_error".into(),
        message: "Request error".into(),
    })?;

    let status = response.status();

    if !status.is_success() {
        let err = response
            .json::<models::ErrorResponse>()
            .await
            .unwrap_or_else(|_| models::ErrorResponse {
                code: status.as_str().to_string(),
                message: format!("Server returned HTTP {}", status),
            });

        return Err(err);
    }

    response
        .json::<T>()
        .await
        .map_err(|_| models::ErrorResponse {
            code: "invalid_response".into(),
            message: "Failed to parse success response".into(),
        })
}

pub(crate) fn build_zip(
    files: &Vec<(String, String)>
) -> (Vec<u8>, models::FileMetadata) {

    let mut zip_bytes = Vec::new();
    let cursor = Cursor::new(&mut zip_bytes);
    let mut zip = ZipWriter::new(cursor);

    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .compression_level(None);

    for (file_name, content) in files {
        zip.start_file(&file_name, options).unwrap();
        zip.write_all(content.as_bytes()).unwrap();
    }

    zip.finish().unwrap();

    let meta = cryptography::get_metadata(&zip_bytes);
    (zip_bytes, meta)
}


pub fn calculate_batch_part_quantity(zip_size_bytes: u64) -> usize {
    if zip_size_bytes <= MAX_PART_SIZE_BYTES {
        return 1;
    }

    ((zip_size_bytes as f64) / (MAX_PART_SIZE_BYTES as f64)).ceil() as usize
}

pub fn split_bytes(input: &[u8], part_count: usize) -> Vec<Vec<u8>> {

    if part_count < 1 {
        return Vec::new();
    }

    let part_size = ((input.len() as f64) / (part_count as f64)).ceil() as usize;
    let mut result = Vec::with_capacity(part_count);

    for i in 0..part_count {
        let start = i * part_size;
        if start >= input.len() {
            break;
        }

        let end = min(start + part_size, input.len());
        result.push(input[start..end].to_vec());
    }

    result
}

pub fn encrypt_and_split(
    zip_bytes: &[u8],
    encryption: &models::EncryptionData,
    part_count: Option<usize>,
) -> Vec<models::BatchPartSendingInfo> {
    let actual_part_count = part_count.unwrap_or_else(|| {
        calculate_batch_part_quantity(zip_bytes.len() as u64)
    });

    let raw_parts = if actual_part_count <= 1 {
        vec![zip_bytes.to_vec()]
    } else {
        split_bytes(zip_bytes, actual_part_count)
    };

    let mut result = Vec::new();

    for (i, part) in raw_parts.into_iter().enumerate() {
        let encrypted = cryptography::encrypt_bytes_with_aes256(
            &part,
            &encryption.cipher_key,
            &encryption.cipher_iv,
        );

        let metadata = cryptography::get_metadata(&encrypted);

        result.push(models::BatchPartSendingInfo {
            data: encrypted,
            metadata,
            ordinal_number: i + 1,
        });
    }

    result
}
