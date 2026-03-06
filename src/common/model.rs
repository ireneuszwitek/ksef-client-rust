use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationStatusInfo {
    #[serde(rename = "code")]
    pub code: i32,

    #[serde(rename = "description")]
    pub description: String,

    #[serde(rename = "details")]
    pub details: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OperationResponse {
    #[serde(rename = "referenceNumber")]
    pub reference_number: String,
}
