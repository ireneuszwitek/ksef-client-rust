use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::time::Duration;

use crate::{auth, certificates, common, cryptography, error, invoice, qr, session, upo};

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

pub struct CompanyInfo {
    pub ksef_token: String,
    pub nip: String,
}

pub struct Client {
    pub(crate) base_url: common::Url,
    qr_url: String,
    pub(crate) sleep_time: u64,
    pub(crate) max_attempts: i32,
    pub(crate) public_certificates: RefCell<Option<Vec<certificates::PemCertificateInfo>>>,
}

impl Client {
    pub fn new(environment: Environment, sleep_time: u64) -> Self {
        let poll_timeout = Duration::from_secs(2 * 60); // 2 minutes
        let total_millis = poll_timeout.as_millis();
        let max_attempts = std::cmp::max(1, (total_millis / sleep_time as u128) as i32);

        Self {
            base_url: common::Url::new(environment.base_url()),
            qr_url: environment.qr_url().to_string(),
            sleep_time,
            max_attempts,
            public_certificates: RefCell::new(None),
        }
    }

    pub async fn get_encryption_data(&self) -> Result<cryptography::EncryptionData, &str> {
        cryptography::get_encryption_data(&self.base_url, &self.public_certificates).await
    }

    pub async fn get_access_tokens(
        &self,
        company_info: &CompanyInfo,
    ) -> Result<auth::token::TokenPair, error::ErrorResponse> {
        common::require_not_empty(&company_info.ksef_token, "company_info.ksef_token")?;
        common::require_not_empty(&company_info.nip, "company_info.nip")?;

        auth::token::get_access_tokens(
            &self.base_url,
            &self.public_certificates,
            &company_info.ksef_token,
            &company_info.nip,
            self.max_attempts,
            self.sleep_time,
        )
        .await
    }

    pub async fn refresh_access_token(
        &self,
        refresh_token: &String,
    ) -> Result<auth::token::TokenInfo, error::ErrorResponse> {
        common::require_not_empty(&refresh_token, "refresh_token")?;

        auth::token::refresh_access_token(&self.base_url, &refresh_token).await
    }

    pub async fn query_invoice_metadata(
        &self,
        request: &invoice::InvoiceQueryFilters,
        access_token: &String,
        page_offset: i32,
        page_size: i32,
        sort_order: invoice::SortOrder,
    ) -> Result<invoice::query::PagedInvoiceResponse, error::ErrorResponse> {
        common::require_not_empty(&access_token, "access_token")?;

        invoice::query::query_metadata(
            &self.base_url,
            &request,
            &access_token,
            page_offset,
            page_size,
            sort_order,
        )
        .await
    }
    pub async fn export_invoice(
        &self,
        filters: &invoice::InvoiceQueryFilters,
        access_token: &String,
    ) -> Result<invoice::export::ExportInvoiceResult, error::ErrorResponse> {
        common::require_not_empty(&access_token, "access_token")?;

        let encryption = match self.get_encryption_data().await {
            Ok(encryption) => encryption,
            Err(e) => {
                return Err(error::ErrorResponse {
                    code: "encryption_error".into(),
                    message: e.into(),
                });
            }
        };

        invoice::export::export_invoice(
            &self.base_url,
            &encryption,
            &filters,
            &access_token,
            self.max_attempts,
            self.sleep_time,
        )
        .await
    }

    pub async fn get_qrcode(
        &self,
        nip: &String,
        issue_date: &DateTime<Utc>,
        invoice_hash: &String,
        resolution_px: Option<u32>,
    ) -> Result<Vec<u8>, error::ErrorResponse> {
        common::require_not_empty(&nip, "nip")?;
        common::require_not_empty(&invoice_hash, "invoice_hash")?;

        let invoice_for_online_url =
            qr::build_invoice_verification_url(&self.qr_url, nip, &issue_date, &invoice_hash)
                .map_err(|_| error::ErrorResponse {
                    code: "build_url_error".into(),
                    message: "Building invoice URL failed".into(),
                })?;

        let png_bytes = qr::generate(&invoice_for_online_url, resolution_px).map_err(|_| {
            error::ErrorResponse {
                code: "qr_generate_error".into(),
                message: "Failed to generate QR".into(),
            }
        })?;

        Ok(png_bytes)
    }

    pub async fn open_online_session(
        &self,
        encryption: &cryptography::EncryptionData,
        access_token: &String,
        system_code: &invoice::SystemCode,
    ) -> Result<session::online::OpenOnlineSessionResponse, error::ErrorResponse> {
        common::require_not_empty(&access_token, "access_token")?;

        session::online::open_online_session(
            &self.base_url,
            &encryption,
            &access_token,
            &system_code,
        )
        .await
    }

    pub async fn close_online_session(
        &self,
        reference_number: &String,
        access_token: &String,
    ) -> Result<(), error::ErrorResponse> {
        common::require_not_empty(&reference_number, "reference_number")?;
        common::require_not_empty(&access_token, "access_token")?;

        session::online::close_online_session(&self.base_url, &reference_number, &access_token)
            .await
    }

    pub async fn send_invoice(
        &self,
        reference_number: &String,
        access_token: &String,
        encryption: &cryptography::EncryptionData,
        xml: &String,
    ) -> Result<common::OperationResponse, error::ErrorResponse> {
        common::require_not_empty(&reference_number, "reference_number")?;
        common::require_not_empty(&access_token, "access_token")?;
        common::require_not_empty(&xml, "xml")?;

        session::online::send_invoice(
            &self.base_url,
            &reference_number,
            &access_token,
            &encryption,
            &xml,
        )
        .await
    }

    pub async fn get_online_session_status(
        &self,
        reference_number: &String,
        access_token: &String,
    ) -> Result<session::status::SessionStatusResponse, error::ErrorResponse> {
        common::require_not_empty(&reference_number, "reference_number")?;
        common::require_not_empty(&access_token, "access_token")?;

        session::online::get_session_status(
            &self.base_url,
            &reference_number,
            &access_token,
            self.max_attempts,
            self.sleep_time,
        )
        .await
    }

    pub async fn send_invoice_batch(
        &self,
        access_token: &String,
        system_code: &invoice::SystemCode,
        list: &Vec<(String, String)>,
        part_count: usize,
    ) -> Result<(String, session::status::SessionStatusResponse), error::ErrorResponse> {
        common::require_not_empty(&access_token, "access_token")?;
        common::vec_require_not_empty(&list, "list")?;

        let encryption = match self.get_encryption_data().await {
            Ok(encryption) => encryption,
            Err(e) => {
                return Err(error::ErrorResponse {
                    code: "encryption_error".into(),
                    message: e.into(),
                });
            }
        };

        session::batch::send_invoice_batch(
            &self.base_url,
            encryption,
            &access_token,
            &system_code,
            &list,
            part_count,
            self.max_attempts,
            self.sleep_time,
        )
        .await
    }

    pub async fn get_session_invoice(
        &self,
        reference_number: &String,
        access_token: &String,
        page_size: i32,
    ) -> Result<session::session_invoice::SessionInvoicesResponse, error::ErrorResponse> {
        common::require_not_empty(&reference_number, "reference_number")?;
        common::require_not_empty(&access_token, "access_token")?;

        session::session_invoice::get_session_invoice(
            &self.base_url,
            &reference_number,
            &access_token,
            page_size,
        )
        .await
    }

    pub async fn get_session_upo(
        &self,
        session_reference_number: &String,
        upo_reference_number: &String,
        access_token: &String,
    ) -> Result<String, error::ErrorResponse> {
        common::require_not_empty(&session_reference_number, "session_reference_number")?;
        common::require_not_empty(&upo_reference_number, "upo_reference_number")?;
        common::require_not_empty(&access_token, "access_token")?;

        session::status::get_session_upo(
            &self.base_url,
            &session_reference_number,
            &upo_reference_number,
            &access_token,
        )
        .await
    }

    pub async fn get_upo(&self, url: &String) -> Result<String, error::ErrorResponse> {
        common::require_not_empty(&url, "url")?;

        upo::get_upo(&url).await
    }


    pub async fn get_sessions(
        &self,
        session_type: session::SessionType,
        access_token: &String,
        page_size: i32,
        session_filter: &Option<session::SessionsFilter>,
    ) -> Result<session::status::SessionsListResponse, error::ErrorResponse> {
        common::require_not_empty(&access_token, "access_token")?;
        
        session::status::get_sessions(&self.base_url, session_type, &access_token, page_size, session_filter).await
    }   
}
