use chrono::DateTime;
use chrono::offset::Utc;
use serde::{Deserialize, Serialize};

use crate::common;

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct AuthenticationChallengeResponse {
    pub(crate) challenge: String,

    pub(crate) timestamp: DateTime<Utc>,

    #[serde(rename = "timestampMs")]
    pub(crate) timestamp_ms: i64,
}

pub(crate) async fn get_auth_challenge(
    base_url: &common::Url,
) -> Result<AuthenticationChallengeResponse, reqwest::Error> {
    let url = "/v2/auth/challenge";

    let reqwest_client = reqwest::Client::new();
    let result = reqwest_client
        .post(base_url.join(url))
        .send()
        .await?
        .json::<AuthenticationChallengeResponse>()
        .await?;
    Ok(result)
}
