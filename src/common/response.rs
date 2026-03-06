use crate::{error, error::ToErrorResponse};
use serde::de::DeserializeOwned;

pub(crate) async fn handle_response<T, R>(
    response: Result<reqwest::Response, reqwest::Error>,
) -> Result<T, error::ErrorResponse>
where
    T: serde::de::DeserializeOwned,
    R: DeserializeOwned + ToErrorResponse,
{
    let response = response.map_err(|_| error::ErrorResponse {
        code: "network_error".into(),
        message: "Network error".into(),
    })?;

    let status = response.status();

    if !status.is_success() {
        let err = match response.json::<R>().await {
            Ok(err) => err,
            Err(_) => {
                return Err(error::ErrorResponse {
                    code: status.as_str().to_string(),
                    message: format!("Server returned HTTP {}", status),
                });
            }
        };
        return Err(err.to_error_response("ksef_api_error".into()));
    }

    response
        .json::<T>()
        .await
        .map_err(|_| error::ErrorResponse {
            code: "invalid_response".into(),
            message: "Failed to parse success response".into(),
        })
}
