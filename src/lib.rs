use base64;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use serde::{Deserialize, Serialize};
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
mod qr;
mod utils;


#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum Environment {
    Prod,
    Test,
    Demo,
}

impl Environment {
    fn base_url(&self) -> &'static str {
        match self {
            Environment::Test => "https://api-test.ksef.mf.gov.pl",
            Environment::Demo => "https://api-demo.ksef.mf.gov.pl",
            Environment::Prod => "https://api.ksef.mf.gov.pl",
        }
    }

    fn qr_url(&self) -> &'static str {
        match self {
            Environment::Test => "https://qr-test.ksef.mf.gov.pl",
            Environment::Demo => "https://qr-demo.ksef.mf.gov.pl",
            Environment::Prod => "https://qr.ksef.mf.gov.pl",
        }
    }
}

pub struct KsefClient {
    base_url: String,
    qr_url: String,
    sleep_time: u64,
    max_attempts: i32,
    public_certificates: RefCell<Option<Vec<models::PemCertificateInfo>>>,
}

pub struct CompanyInfo {
    pub ksef_token: String,
    pub nip: String,
}

impl KsefClient {
    pub fn new(environment: Environment, sleep_time: u64) -> Self {

        let poll_timeout = Duration::from_secs(2 * 60); // 2 minutes
        let total_millis = poll_timeout.as_millis();
        let max_attempts = std::cmp::max(1, (total_millis / sleep_time as u128) as i32);

        Self {
            base_url: environment.base_url().to_string(),
            qr_url: environment.qr_url().to_string(),
            sleep_time,
            max_attempts,
            public_certificates: RefCell::new(None),
        }
    }

    fn join_url(&self, url: &str) -> String {
        format!("{}{}", self.base_url, url)
    }

    pub async fn get_encryption_data(&self) -> Result<models::EncryptionData, &str> {
        cryptography::get_encryption_data(&self).await
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

        let _ = utils::pool(
            || {
                self.get_auth_status(
                    &signature.reference_number,
                    &signature.authentication_token.token,
                )
            },
            |result| result.status.code == 200,
            self.max_attempts,
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
    ) -> Result<models::AuthStatus, models::ErrorResponse> {
        let escaped = encode(auth_operation_reference_number);
        let url = format!("/v2/auth/{}", escaped);

        let reqwest_client = reqwest::Client::new();
        let response = reqwest_client
            .get(self.join_url(url.as_str()))
            .bearer_auth(&authentication_token)
            .send()
            .await;

        utils::handle_response::<models::AuthStatus>(response).await
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

    async fn start_export_invoices(
        &self,
        request: &invoice::ExportInvoiceRequest,
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

    async fn get_export_invoice_status(
        &self,
        reference_number: &String,
        access_token: &String,
    ) -> Result<invoice::ExportInvoiceStatusResponse, models::ErrorResponse> {
        let url = format!("/v2/invoices/exports/{}", encode(reference_number));

        let reqwest_client = reqwest::Client::new();
        let response = reqwest_client
            .get(self.join_url(url.as_str()))
            .bearer_auth(&access_token)
            .send()
            .await;

        utils::handle_response::<invoice::ExportInvoiceStatusResponse>(response).await
    }

    pub async fn export_invoice(
        &self,
        filters: &invoice::InvoiceQueryFilters,
        access_token: &String,
    ) -> Result<invoice::ExportInvoiceResult, models::ErrorResponse> {
        let encryption = match self.get_encryption_data().await {
            Ok(encryption) => encryption,
            Err(e) => {
                return Err(models::ErrorResponse {
                    code: "encryption_error".into(),
                    message: e.into(),
                });
            }
        };

        let export_invoice_request = invoice::ExportInvoiceRequest {
            encryption: encryption.encryption_info.clone(),
            filters: (*filters).clone(),
        };

        let start_export_invoices = match self
            .start_export_invoices(&export_invoice_request, &access_token)
            .await
        {
            Ok(start_export_invoices) => start_export_invoices,
            Err(e) => {
                return Err(models::ErrorResponse {
                    code: "start_export_invoices_error".into(),
                    message: format!("Status: {}", e),
                });
            }
        };

        let export_invoice_status = match utils::pool(
            || {
                self.get_export_invoice_status(
                    &start_export_invoices.reference_number,
                    &access_token,
                )
            },
            |result| result.status.code == 200,
            self.max_attempts,
            self.sleep_time,
        )
        .await
        {
            Ok(export_status) => export_status,
            Err(e) => {
                return Err(models::ErrorResponse {
                    code: "export_invoice_status_error".into(),
                    message: e.into(),
                });
            }
        };

        let mut metadata_summaries: Vec<invoice::InvoiceSummary> = Vec::new();
        let mut xml_files: HashMap<String, String> = HashMap::new();

        if !export_invoice_status.package.parts.is_empty() {
            let decrypted_archive_stream = match self
                .download_package_parts(&export_invoice_status.package.parts, &encryption)
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

        let result = invoice::ExportInvoiceResult {
            metadata_summaries: metadata_summaries,
            xml_files: xml_files,
            is_truncated: export_invoice_status.package.is_truncated,
            last_permanent_storage_date: export_invoice_status.package.last_permanent_storage_date,
            permanent_storage_hwm_date: export_invoice_status.package.permanent_storage_hwm_date,
        };

        Ok(result)
    }

    async fn download_package_parts(
        &self,
        parts: &Vec<invoice::ExportInvoicePackagePart>,
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
        part: &invoice::ExportInvoicePackagePart,
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
        nip: &String,
        issue_date: &DateTime<Utc>,
        invoice_hash: &String,
        resolution_px: Option<u32>,
    ) -> Result<Vec<u8>, models::ErrorResponse> {
        let invoice_for_online_url =
            qr::build_invoice_verification_url(&self.qr_url, nip, &issue_date, &invoice_hash)
                .map_err(|_| models::ErrorResponse {
                    code: "build_url_error".into(),
                    message: "Building invoice URL failed".into(),
                })?;

        let png_bytes = qr::generate(&invoice_for_online_url, resolution_px).map_err(|_| {
            models::ErrorResponse {
                code: "qr_generate_error".into(),
                message: "Failed to generate QR".into(),
            }
        })?;

        Ok(png_bytes)
    }

    pub async fn open_online_session(
        &self,
        encryption: &models::EncryptionData,
        access_token: &String,
        system_code: &invoice::SystemCode,
    ) -> Result<models::OpenOnlineSessionResponse, models::ErrorResponse> {
        let form_code = models::FormCode {
            system_code: system_code.system_code().into(),
            schema_version: system_code.schema_version().into(),
            value: system_code.value().into(),
        };

        let request = models::OpenOnlineSessionRequest {
            form_code,
            encryption: encryption.encryption_info.clone(),
        };

        let url = "/v2/sessions/online";

        let reqwest_client = reqwest::Client::new();
        let response = reqwest_client
            .post(self.join_url(url))
            .json(&request)
            .bearer_auth(&access_token)
            .send()
            .await;

        utils::handle_response::<models::OpenOnlineSessionResponse>(response).await
    }

    pub async fn close_online_session(
        &self,
        reference_number: &String,
        access_token: &String,
    ) -> Result<(), models::ErrorResponse> {
        let url = format!("/v2/sessions/online/{}/close", encode(reference_number));

        let reqwest_client = reqwest::Client::new();
        let response = reqwest_client
            .post(self.join_url(url.as_str()))
            .bearer_auth(&access_token)
            .send()
            .await
            .map_err(|_| models::ErrorResponse {
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

        Ok(())
    }

    pub async fn send_invoice(
        &self,
        reference_number: &String,
        access_token: &String,
        encryption: &models::EncryptionData,
        xml: &String,
    ) -> Result<invoice::OperationResponse, models::ErrorResponse> {
        let invoice = xml.as_bytes().to_vec();

        let encrypted_invoice = cryptography::encrypt_bytes_with_aes256(
            &invoice,
            &encryption.cipher_key,
            &encryption.cipher_iv,
        );

        let invoice_metadata = cryptography::get_metadata(&invoice);
        let encrypted_metadata = cryptography::get_metadata(&encrypted_invoice);

        let request = invoice::SendInvoiceRequest {
            invoice_hash: invoice_metadata.hash_sha,
            invoice_size: invoice_metadata.file_size as i64,
            encrypted_invoice_hash: encrypted_metadata.hash_sha,
            encrypted_invoice_size: encrypted_metadata.file_size as i64,
            encrypted_invoice_content: STANDARD.encode(encrypted_invoice),
            offline_mode: false,
            hash_of_corrected_invoice: None,
        };

        let url = format!("/v2/sessions/online/{}/invoices", encode(reference_number));

        let reqwest_client = reqwest::Client::new();
        let response = reqwest_client
            .post(self.join_url(url.as_str()))
            .json(&request)
            .bearer_auth(&access_token)
            .send()
            .await;

        utils::handle_response::<invoice::OperationResponse>(response).await
    }

    async fn get_session_status(
        &self,
        reference_number: &String,
        access_token: &String,
    ) -> Result<models::SessionStatusResponse, models::ErrorResponse> {
        let url = format!("/v2/sessions/{}", encode(reference_number));

        let reqwest_client = reqwest::Client::new();
        let response = reqwest_client
            .get(self.join_url(url.as_str()))
            .bearer_auth(&access_token)
            .send()
            .await;

        utils::handle_response::<models::SessionStatusResponse>(response).await
    }

    async fn try_get_session_status(
        &self,
        reference_number: &String,
        access_token: &String,
    ) -> Result<models::SessionStatusResponse, models::ErrorResponse> {
        let mut attempt = 0;

        loop {
            if let Ok(status_response) = self
                .get_session_status(&reference_number, &access_token)
                .await
            {
                if status_response.successful_invoice_count.is_some() {
                    return Ok(status_response);
                }

                if attempt >= self.max_attempts {
                    return Ok(status_response);
                }
            }

            attempt += 1;
            tokio::time::sleep(tokio::time::Duration::from_millis(self.sleep_time)).await;
        }
    }

    pub async fn get_online_session_status(
        &self,
        reference_number: &String,
        access_token: &String,
    ) -> Result<models::SessionStatusResponse, models::ErrorResponse> {
        let session_status = match utils::pool(
            || self.try_get_session_status(&reference_number, &access_token),
            |result| result.invoice_count == result.successful_invoice_count,
            self.max_attempts,
            self.sleep_time,
        )
        .await
        {
            Ok(session_status) => session_status,
            Err(_) => {
                return Err(models::ErrorResponse {
                    code: "get_session_status_error".into(),
                    message: "get_session_status_error".into(), //e.into(),
                });
            }
        };

        Ok(session_status)
    }

    pub async fn open_batch_session(
        &self,
        request: &models::OpenBatchSessionRequest,
        access_token: &String,
    ) -> Result<models::OpenBatchSessionResponse, models::ErrorResponse> {
        let url = "/v2/sessions/batch";

        let reqwest_client = reqwest::Client::new();
        let response = reqwest_client
            .post(self.join_url(url))
            .json(&request)
            .bearer_auth(&access_token)
            .send()
            .await;

        utils::handle_response::<models::OpenBatchSessionResponse>(response).await
    }

    async fn send_package_parts(
        &self,
        parts: &Vec<models::PackagePartSignatureInitResponseType>,
        batch_part_sending_infos: &Vec<models::BatchPartSendingInfo>,
    ) -> Result<(), models::ErrorResponse> {
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
            return Err(models::ErrorResponse {
                code: "send_package_parts_error".into(),
                message: errors.join("\n"),
            });
        }

        Ok(())
    }

    async fn close_batch_session(
        &self,
        reference_number: &String,
        access_token: &String,
    ) -> Result<bool, models::ErrorResponse> {
        let url = format!("/v2/sessions/batch/{}/close", encode(reference_number));

        let reqwest_client = reqwest::Client::new();
        let response = reqwest_client
            .post(self.join_url(url.as_str()))
            .bearer_auth(&access_token)
            .send()
            .await
            .map_err(|_| models::ErrorResponse {
                code: "request_error".into(),
                message: "Request error".into(),
            })?;

        let status = response.status();

        if !status.is_success() {
            return Err(models::ErrorResponse {
                code: status.as_str().to_string(),
                message: format!("Server returned HTTP {}", status),
            });
        }

        Ok(true)
    }

    async fn get_batch_session_status(
        &self,
        reference_number: &String,
        access_token: &String,
    ) -> Result<models::SessionStatusResponse, models::ErrorResponse> {
        let session_status = match utils::pool(
            || self.try_get_session_status(&reference_number, &access_token),
            |result| result.status.code == 200,
            self.max_attempts,
            self.sleep_time,
        )
        .await
        {
            Ok(session_status) => session_status,
            Err(_) => {
                return Err(models::ErrorResponse {
                    code: "get_session_status_error".into(),
                    message: "get_session_status_error".into(), //e.into(),
                });
            }
        };

        Ok(session_status)
    }

    pub async fn send_invoice_batch(
        &self,
        access_token: &String,
        system_code: &invoice::SystemCode,
        list: &Vec<(String, String)>,
        part_count: usize,
    ) -> Result<models::SessionStatusResponse, models::ErrorResponse> {
        // Generate encryption
        let encryption = match self.get_encryption_data().await {
            Ok(encryption) => encryption,
            Err(e) => {
                return Err(models::ErrorResponse {
                    code: "encryption_error".into(),
                    message: e.into(),
                });
            }
        };

        let form_code = models::FormCode {
            system_code: system_code.system_code().into(),
            schema_version: system_code.schema_version().into(),
            value: system_code.value().into(),
        };

        let (zip_bytes, zip_meta) = utils::build_zip(&list);

        let encrypted_parts = utils::encrypt_and_split(&zip_bytes, &encryption, Some(part_count));

        if encrypted_parts.len() < 1 {
            return Err(models::ErrorResponse {
                code: "part_size_error".into(),
                message: "the number of parts is less than 1".into(),
            });
        }

        let parts: Vec<_> = encrypted_parts
            .iter()
            .map(|p| models::BatchFilePartInfo {
                ordinal_number: p.ordinal_number,
                file_size: p.metadata.file_size,
                file_hash: p.metadata.hash_sha.clone(),
            })
            .collect();

        let open_batch_request = models::OpenBatchSessionRequest {
            form_code: form_code,
            batch_file: models::BatchFileInfo {
                file_size: zip_meta.file_size,
                file_hash: zip_meta.hash_sha,
                file_parts: parts,
            },
            encryption: encryption.encryption_info,
            offline_mode: false,
        };

        let open_batch_session_response = match self
            .open_batch_session(&open_batch_request, access_token)
            .await
        {
            Ok(open_batch_session_response) => open_batch_session_response,
            Err(e) => {
                return Err(models::ErrorResponse {
                    code: "open_batch_session".into(),
                    message: e.message,
                });
            }
        };

        if let Err(e) = self
            .send_package_parts(
                &open_batch_session_response.part_upload_requests,
                &encrypted_parts,
            )
            .await
        {
            return Err(models::ErrorResponse {
                code: "open_batch_session".into(),
                message: e.message,
            });
        };

        // close session
        let _ = utils::pool(
            || {
                self.close_batch_session(
                    &open_batch_session_response.reference_number,
                    &access_token,
                )
            },
            |result| *result,
            self.max_attempts,
            self.sleep_time,
        )
        .await;

        let session_status = match self
            .get_batch_session_status(&open_batch_session_response.reference_number, &access_token)
            .await
        {
            Ok(online_session_status) => online_session_status,
            Err(e) => return Err(e),
        };

        Ok(session_status)
    }
}
