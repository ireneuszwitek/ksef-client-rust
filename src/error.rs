use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Clone)]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
}

pub trait ToErrorResponse {
    fn to_error_response(&self, code: String) -> ErrorResponse;
}

impl ToErrorResponse for ErrorResponse {
    fn to_error_response(&self, _: String) -> ErrorResponse {
        self.clone()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiErrorResponse {
    #[serde(rename = "exception")]
    pub exception: ApiExceptionContent,
}

impl ApiErrorResponse {
    pub(crate) fn build_error_message_from_details(&self) -> String {
        let details_list = match self.exception.exception_detail_list.as_slice() {
            [] => return String::new(),
            list => list,
        };

        let parts: Vec<String> = details_list
            .iter()
            .map(|detail| {
                let details_text = match &detail.details {
                    Some(list) if !list.is_empty() => list.join("; "),
                    _ => String::new(),
                };

                if details_text.is_empty() {
                    format!(
                        "{}: {}",
                        detail.exception_code, detail.exception_description
                    )
                } else {
                    format!(
                        "{}: {} - {}",
                        detail.exception_code, detail.exception_description, details_text
                    )
                }
            })
            .collect();

        parts.join(" | ")
    }
}

impl ToErrorResponse for ApiErrorResponse {
    fn to_error_response(&self, code: String) -> ErrorResponse {
        ErrorResponse {
            code: code,
            message: self.build_error_message_from_details(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiExceptionContent {
    #[serde(rename = "exceptionDetailList")]
    pub exception_detail_list: Vec<ApiExceptionDetail>,

    #[serde(rename = "serviceCode")]
    pub service_code: String,

    #[serde(rename = "timestamp")]
    pub timestamp: String,

    #[serde(rename = "serviceName", default)]
    pub service_name: String,

    #[serde(rename = "referenceNumber", default)]
    pub reference_number: String,

    #[serde(rename = "serviceCtx", default)]
    pub service_ctx: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiExceptionDetail {
    #[serde(rename = "exceptionCode")]
    pub exception_code: i32,

    #[serde(rename = "exceptionDescription")]
    pub exception_description: String,

    #[serde(rename = "details")]
    pub details: Option<Vec<String>>,
}
