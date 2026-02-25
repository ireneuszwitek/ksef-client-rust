use base64;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;

use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{Cursor, Write};
use std::time::Duration;

use urlencoding::encode;

use chrono::{DateTime, Utc};

const METADATA_ENTRY_NAME: &str = "_metadata.json";
const XML_FILE_EXTENSION: &str = ".xml";

mod certificates;
mod cryptography;
pub mod invoice;
mod models;
mod qrcode;
mod utils;

pub struct KsefClient {
    base_url: String,
    sleep_time: u64,
    public_certificates: RefCell<Option<Vec<models::PemCertificateInfo>>>,
}

pub struct CompanyInfo {
    pub ksef_token: String,
    pub nip: String,
}

impl KsefClient {
    pub fn new(base_url: String, sleep_time: u64) -> Result<Self, url::ParseError> {
        let base_url_parsed = url::Url::parse(&base_url)?;
        let base_url = base_url_parsed
            .to_string()
            .trim_end_matches('/')
            .to_string();
        Ok(Self {
            base_url,
            sleep_time,
            public_certificates: RefCell::new(None),
        })
    }

    fn join_url(&self, url: &str) -> String {
        format!("{}{}", self.base_url, url)
    }

    pub async fn get_access_tokens(
        &self,
        company_info: &CompanyInfo,
    ) -> Result<models::TokenPair, &str> {
        let ksef_token_cert = match certificates::public_certificate(
            &self,
            &models::PublicKeyCertificateUsage::KsefTokenEncryption,
        )
        .await
        {
            Ok(ksef_token_cert) => ksef_token_cert,
            Err(e) => {
                return Err(e);
            }
        };

        let challenge = match self.get_auth_challenge().await {
            Ok(challenge) => challenge,
            Err(_) => {
                return Err("challenge_error");
            }
        };

        let timestamp_ms = challenge.timestamp.timestamp_millis();

        let token_with_timestamp = format!("{}|{}", &company_info.ksef_token, timestamp_ms);
        let token_bytes: Vec<u8> = token_with_timestamp.as_bytes().to_vec();
        let encrypted: Vec<u8> = cryptography::encrypt_ksef_token_with_rsa_using_public_key(
            &ksef_token_cert,
            &token_bytes,
        )
        .unwrap();

        let encrypted_token_b64 = STANDARD.encode(&encrypted);

        let request = models::AuthenticationKsefTokenRequest {
            challenge: challenge.challenge,
            context_identifier: models::AuthenticationTokenContextIdentifier {
                auth_type: models::AuthenticationTokenContextIdentifierType::Nip,
                value: Some(company_info.nip.clone()),
            },
            encrypted_token: encrypted_token_b64,
        };

        let signature = match self.submit_ksef_token_auth_request(&request).await {
            Ok(signature) => signature,
            Err(_) => {
                return Err("signature_error");
            }
        };

        let poll_timeout = Duration::from_secs(2 * 60); // 2 minuty
        let total_millis = poll_timeout.as_millis();
        let max_attempts = std::cmp::max(1, (total_millis / self.sleep_time as u128) as i32);

        let _ = utils::pool(
            || {
                self.get_auth_status(
                    &signature.reference_number,
                    &signature.authentication_token.token,
                )
            },
            |result| result.status.code == 200,
            max_attempts,
            self.sleep_time,
        )
        .await;

        let tokens = match self
            .get_access_token_by_authentication_token(&signature.authentication_token.token)
            .await
        {
            Ok(tokens) => tokens,
            Err(_) => {
                return Err("token_error");
            }
        };

        Ok(tokens)
    }

    pub async fn refresh_access_token(
        &self,
        refresh_token: &String,
    ) -> Result<models::TokenInfo, &str> {
        let url = "/v2/auth/token/refresh";

        let reqwest_client = reqwest::Client::new();
        let resp = reqwest_client
            .post(self.join_url(url))
            .bearer_auth(&refresh_token)
            .send()
            .await
            .map_err(|_| "network error")?;

        if resp.status().is_success() {
            let result = resp
                .json::<models::RefreshTokenResponse>()
                .await
                .map_err(|_| "invalid success response")?;
            return Ok(result.access_token);
        }

        Err("server returned error status")
    }

    pub async fn query_invoice_metadata(
        &self,
        request: &invoice::InvoiceQueryFilters,
        access_token: &String,
        page_offset: i32,
        page_size: i32,
        sort_order: invoice::SortOrder,
    ) -> Result<invoice::PagedInvoiceResponse, models::ErrorResponse> {
        let mut url = format!("/v2/invoices/query/metadata?sortOrder={}", sort_order);

        if page_offset > 0 {
            url = format!("{}&pageOffset={}", url, page_offset);
        }

        if page_size > 0 {
            url = format!("{}&pageSize={}", url, page_size);
        }

        let reqwest_client = reqwest::Client::new();
        let resp = reqwest_client
            .post(self.join_url(url.as_str()))
            .bearer_auth(access_token)
            .header("Content-Type", "application/json")
            .json(request)
            .send()
            .await
            .map_err(|_| models::ErrorResponse {
                code: "network_error".into(),
                message: "Failed to send request".into(),
            })?;

        let status = resp.status();

        if status.is_success() {
            let ok = resp
                .json::<invoice::PagedInvoiceResponse>()
                .await
                .map_err(|_| models::ErrorResponse {
                    code: "invalid_response".into(),
                    message: "Failed to parse success response".into(),
                })?;
            return Ok(ok);
        }

        let err = resp
            .json::<models::ErrorResponse>()
            .await
            .unwrap_or_else(|_| models::ErrorResponse {
                code: "unknown_error".into(),
                message: format!("Server returned HTTP {}", status),
            });

        Err(err)
    }

    async fn get_auth_challenge(
        &self,
    ) -> Result<models::AuthenticationChallengeResponse, reqwest::Error> {
        let url = "/v2/auth/challenge";

        let reqwest_client = reqwest::Client::new();
        let result = reqwest_client
            .post(self.join_url(url))
            .send()
            .await?
            .json::<models::AuthenticationChallengeResponse>()
            .await?;
        Ok(result)
    }

    async fn submit_ksef_token_auth_request(
        &self,
        request: &models::AuthenticationKsefTokenRequest,
    ) -> Result<models::SignatureResponse, reqwest::Error> {
        let url = "/v2/auth/ksef-token";

        let reqwest_client = reqwest::Client::new();
        let result = reqwest_client
            .post(self.join_url(url))
            .json(&request)
            .send()
            .await?
            .json::<models::SignatureResponse>()
            .await?;
        Ok(result)
    }

    async fn get_auth_status(
        &self,
        auth_operation_reference_number: &String,
        authentication_token: &String,
    ) -> Result<models::AuthStatus, reqwest::Error> {
        let escaped = encode(auth_operation_reference_number);
        let url = format!("/v2/auth/{}", escaped);

        let reqwest_client = reqwest::Client::new();
        let result = reqwest_client
            .get(self.join_url(url.as_str()))
            .bearer_auth(&authentication_token)
            .send()
            .await?
            .json::<models::AuthStatus>()
            .await?;
        Ok(result)
    }

    async fn get_access_token_by_authentication_token(
        &self,
        authentication_token: &String,
    ) -> Result<models::TokenPair, reqwest::Error> {
        let url = "/v2/auth/token/redeem";

        let reqwest_client = reqwest::Client::new();
        let result = reqwest_client
            .post(self.join_url(url))
            .bearer_auth(&authentication_token)
            .send()
            .await?
            .json::<models::TokenPair>()
            .await?;
        Ok(result)
    }

    async fn start_invoices_export(
        &self,
        request: &invoice::InvoiceExportRequest,
        access_token: &String,
    ) -> Result<invoice::OperationResponse, reqwest::Error> {
        let url = "/v2/invoices/exports";

        let reqwest_client = reqwest::Client::new();
        let result = reqwest_client
            .post(self.join_url(url))
            .json(&request)
            .bearer_auth(&access_token)
            .send()
            .await?
            .json::<invoice::OperationResponse>()
            .await?;
        Ok(result)
    }

    async fn get_invoice_export_status(
        &self,
        reference_number: &String,
        access_token: &String,
    ) -> Result<invoice::InvoiceExportStatusResponse, reqwest::Error> {
        let url = format!("/v2/invoices/exports/{}", encode(reference_number));

        let reqwest_client = reqwest::Client::new();
        let result = reqwest_client
            .get(self.join_url(url.as_str()))
            .bearer_auth(&access_token)
            .send()
            .await?
            .json::<invoice::InvoiceExportStatusResponse>()
            .await?;
        Ok(result)
    }

    pub async fn invoice_export(
        &self,
        filters: &invoice::InvoiceQueryFilters,
        access_token: &String,
    ) -> Result<invoice::InvoiceExportResult, models::ErrorResponse> {
        let encryption = match cryptography::get_encryption_data(&self).await {
            Ok(encryption) => encryption,
            Err(e) => {
                return Err(models::ErrorResponse {
                    code: "encryption_error".into(),
                    message: e.into(),
                });
            }
        };

        let invoice_export_request = invoice::InvoiceExportRequest {
            encryption: encryption.encryption_info.clone(),
            filters: (*filters).clone(),
        };

        let start_invoices_export = match self
            .start_invoices_export(&invoice_export_request, &access_token)
            .await
        {
            Ok(start_invoices_export) => start_invoices_export,
            Err(e) => {
                return Err(models::ErrorResponse {
                    code: "start_invoices_export_error".into(),
                    message: format!("Status: {}", e),
                });
            }
        };

        let poll_timeout = Duration::from_secs(2 * 60); // 2 minuty
        let total_millis = poll_timeout.as_millis();
        let max_attempts = std::cmp::max(1, (total_millis / self.sleep_time as u128) as i32);

        let invoice_export_status = match utils::pool(
            || {
                self.get_invoice_export_status(
                    &start_invoices_export.reference_number,
                    &access_token,
                )
            },
            |result| result.status.code == 200,
            max_attempts,
            self.sleep_time,
        )
        .await
        {
            Ok(export_status) => export_status,
            Err(e) => {
                return Err(models::ErrorResponse {
                    code: "invoice_export_status_error".into(),
                    message: e.into(),
                });
            }
        };

        let mut metadata_summaries: Vec<invoice::InvoiceSummary> = Vec::new();
        let mut xml_files: HashMap<String, String> = HashMap::new();

        if !invoice_export_status.package.parts.is_empty() {
            let decrypted_archive_stream = match self
                .download_package_parts(&invoice_export_status.package.parts, &encryption)
                .await
            {
                Ok(decrypted_archive_stream) => decrypted_archive_stream,
                Err(e) => {
                    return Err(models::ErrorResponse {
                        code: "download_package_parts_error".into(),
                        message: e.into(),
                    });
                }
            };

            let unzipped_files = utils::unzip(decrypted_archive_stream);

            for (file_name, content) in unzipped_files {
                if file_name.eq_ignore_ascii_case(METADATA_ENTRY_NAME) {
                    if let Ok(metadata) =
                        serde_json::from_str::<invoice::InvoicePackageMetadata>(&content)
                    {
                        if let Some(invoices) = metadata.invoices {
                            metadata_summaries.extend(invoices);
                        }
                    }
                } else if file_name.to_lowercase().ends_with(XML_FILE_EXTENSION) {
                    xml_files.insert(file_name.to_lowercase(), content);
                }
            }
        }

        let result = invoice::InvoiceExportResult {
            metadata_summaries: metadata_summaries,
            xml_files: xml_files,
            is_truncated: invoice_export_status.package.is_truncated,
            last_permanent_storage_date: invoice_export_status.package.last_permanent_storage_date,
            permanent_storage_hwm_date: invoice_export_status.package.permanent_storage_hwm_date,
        };

        Ok(result)
    }

    async fn download_package_parts(
        &self,
        parts: &Vec<invoice::InvoiceExportPackagePart>,
        encryption: &models::EncryptionData,
    ) -> Result<Cursor<Vec<u8>>, &str> {
        let mut buffer = Cursor::new(Vec::new());

        let mut parts_sorted: Vec<_> = parts.iter().collect();
        parts_sorted.sort_by_key(|p| p.ordinal_number);

        for part in parts_sorted {
            let encrypted_bytes = match self.download_package_part(&part).await {
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

    async fn download_package_part(
        &self,
        part: &invoice::InvoiceExportPackagePart,
    ) -> Result<Vec<u8>, &str> {
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

    pub async fn get_qrcode(
        &self,
        base_url: &String,
        nip: &String,
        issue_date: &DateTime<Utc>,
        invoice_hash: &String,
        resolution_px: Option<i32>,
    ) -> Result<Vec<u8>, models::ErrorResponse> {
        let invoice_for_online_url =
            qrcode::build_invoice_verification_url(&base_url, nip, &issue_date, &invoice_hash)
                .map_err(|_| models::ErrorResponse {
                    code: "build_url_error".into(),
                    message: "Building invoice URL failed".into(),
                })?;

        let png_bytes = qrcode::generate(&invoice_for_online_url, resolution_px).map_err(|_| {
            models::ErrorResponse {
                code: "qr_generate_error".into(),
                message: "QR generation failed".into(),
            }
        })?;

        Ok(png_bytes)
    }
}
