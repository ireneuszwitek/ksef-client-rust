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


#[cfg(test)]
mod tests {
    use super::*;
    use crate::invoice;

    fn fake_response(status: u16, body: &str) -> reqwest::Response {
        let response = http::Response::builder()
            .status(status)
            .body(body.to_string())
            .unwrap();

        reqwest::Response::from(response)
    }

    #[tokio::test]
    async fn test_handle_response_network_error() {
        // Port 9 is almost always closed → guaranteed network error
        let response = reqwest::get("http://127.0.0.1:9").await;

        assert!(response.is_err());

        let result = handle_response::<(), error::ErrorResponse>(response).await;

        assert!(matches!(
            result,
            Err(error::ErrorResponse { code, .. }) if code == "network_error"
        ));
    }

    #[tokio::test]
    async fn test_handle_response_http_error_with_json_for_error_response() {
        let body = r#"{"code": "bad", "message": "Invalid"}"#;
        let resp = Ok(fake_response(400, body));

        let result = handle_response::<(), error::ErrorResponse>(resp).await;

        assert!(matches!(result,Err(error::ErrorResponse { code, .. }) if code == "bad"));

    }

    #[tokio::test]
    async fn test_handle_response_http_error_with_json_for_api_error_response() {
        let body = r#"{"code": "bad", "message": "Invalid"}"#;
        let resp = Ok(fake_response(400, body));

        let result = handle_response::<(), error::ApiErrorResponse>(resp).await;

        assert!(matches!(result,Err(error::ErrorResponse { code, .. }) if code == "400"));

    }

    #[tokio::test]
    async fn test_handle_response_http_error_invalid_json() {
        let resp = Ok(fake_response(500, "not json"));
        let result = handle_response::<(), error::ErrorResponse>(resp).await;

        assert!(matches!(result, Err(error::ErrorResponse { code, .. }) if code == "500"));
    }

    #[tokio::test]
    async fn test_handle_response_success() {
        let body = r#"{"code": 123, "description" : "abcd"}"#;
        let resp = Ok(fake_response(200, body));
        let result = handle_response::<invoice::InvoiceStatusInfo, error::ErrorResponse>(resp).await;

        assert!(result.is_ok());
    }
    #[tokio::test]
    async fn test_handle_response_invalid_success_json() {
        let resp = Ok(fake_response(200, "not json"));

        let result = handle_response::<invoice::InvoiceStatusInfo, error::ErrorResponse>(resp).await;

        assert!(matches!(result, Err(error::ErrorResponse { code, .. }) if code == "invalid_response"));
    }

}

