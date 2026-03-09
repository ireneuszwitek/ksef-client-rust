use crate::{error};

pub(crate) async fn get_upo(
    url: &String,
) -> Result<String, error::ErrorResponse> {

    let reqwest_client = reqwest::Client::new();
    let response = reqwest_client
        .get(url)
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
