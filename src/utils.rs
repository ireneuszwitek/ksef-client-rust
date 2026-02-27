use std::collections::HashMap;
use std::io::{Read, Seek};
use tokio::time::{Duration, sleep};
use zip::ZipArchive;
use crate::models;


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
