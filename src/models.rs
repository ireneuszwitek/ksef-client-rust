use serde::{Deserialize, Serialize};
use chrono::{DateTime, FixedOffset};
use chrono::offset::Utc;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EncryptionInfo {
    #[serde(rename = "EncryptedSymmetricKey")]
    pub encrypted_symmetric_key: String,

    #[serde(rename = "InitializationVector")]
    pub initialization_vector: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EncryptionData {
    #[serde(rename = "CipherKey")]
    pub cipher_key: Vec<u8>,

    #[serde(rename = "CipherIv")]
    pub cipher_iv: Vec<u8>,

    #[serde(rename = "EncryptionInfo")]
    pub encryption_info: EncryptionInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct PemCertificateInfo {
    #[serde(rename = "certificate")]
    pub(crate) certificate: String,

    #[serde(rename = "validFrom")]
    pub(crate) valid_from: DateTime<FixedOffset>,

    #[serde(rename = "validTo")]
    pub(crate) valid_to: DateTime<FixedOffset>,

    #[serde(rename = "usage")]
    pub(crate)usage: Vec<PublicKeyCertificateUsage>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) enum PublicKeyCertificateUsage {
    #[serde(rename = "KsefTokenEncryption")]
    KsefTokenEncryption,

    #[serde(rename = "SymmetricKeyEncryption")]
    SymmetricKeyEncryption,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct AuthenticationChallengeResponse {
    pub(crate) challenge: String,

    pub(crate) timestamp: DateTime<Utc>,

    #[serde(rename = "timestampMs")]
    pub(crate) timestamp_ms: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct AuthenticationKsefTokenRequest {
    #[serde(rename = "Challenge")]
    pub(crate) challenge: String,
    #[serde(rename = "ContextIdentifier")]
    pub(crate) context_identifier: AuthenticationTokenContextIdentifier,
    #[serde(rename = "EncryptedToken")]
    pub(crate) encrypted_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct AuthenticationTokenContextIdentifier {
    #[serde(rename = "Type")]
    pub(crate) auth_type: AuthenticationTokenContextIdentifierType,
    #[serde(rename = "Value")]
    pub(crate) value: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum AuthenticationTokenContextIdentifierType {
    #[serde(rename = "Nip")]
    Nip,

    #[serde(rename = "InternalId")]
    InternalId,

    #[serde(rename = "NipVatUe")]
    NipVatUe,

    #[serde(rename = "PeppolId")]
    PeppolId,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct OperationToken {
    #[serde(rename = "token")]
    pub(crate) token: String,

    #[serde(rename = "validUntil")]
    pub(crate) valid_until: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct SignatureResponse {
    #[serde(rename = "referenceNumber")]
    pub(crate) reference_number: String,

    #[serde(rename = "authenticationToken")]
    pub(crate) authentication_token: OperationToken,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationStatusInfo {
    #[serde(rename = "code")]
    pub code: i32,

    #[serde(rename = "description")]
    pub description: String,
    
    #[serde(rename = "details")] 
    pub details: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct AuthStatus {
    #[serde(rename = "startDate")]
    pub(crate) start_date: DateTime<Utc>,

    #[serde(rename = "authenticationMethod")]
    pub(crate) authentication_method: AuthenticationMethodEnum,

    #[serde(rename = "status")]
    pub(crate) status: OperationStatusInfo,

    #[serde(rename = "isTokenRedeemed")]
    pub(crate) is_token_redeemed: Option<bool>,

    #[serde(rename = "lastTokenRefreshDate")]
    pub(crate) last_token_refresh_date: Option<DateTime<Utc>>,

    #[serde(rename = "refreshTokenValidUntil")]
    pub(crate) refresh_token_valid_until: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) enum AuthenticationMethodEnum {
    #[serde(rename = "Token")]
    Token,

    #[serde(rename = "TrustedProfile")]
    TrustedProfile,

    #[serde(rename = "InternalCertificate")]
    InternalCertificate,

    #[serde(rename = "QualifiedSignature")]
    QualifiedSignature,

    #[serde(rename = "QualifiedSeal")]
    QualifiedSeal,

    #[serde(rename = "PersonalSignature")]
    PersonalSignature,

    #[serde(rename = "PeppolSignature")]
    PeppolSignature,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenInfo {
    #[serde(rename = "token")]
    pub token: String,

    #[serde(rename = "validUntil")]
    pub valid_until: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPair {
    #[serde(rename = "accessToken")]
    pub access_token: TokenInfo,

    #[serde(rename = "refreshToken")]
    pub refresh_token: TokenInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct RefreshTokenResponse {
    #[serde(rename = "accessToken")]
    pub(crate) access_token: TokenInfo,
}

// Sessions

#[derive(Debug, Serialize, Deserialize)]
pub struct FormCode {
    #[serde(rename = "systemCode")]
    pub system_code: String,

    #[serde(rename = "schemaVersion")]
    pub schema_version: String,

    #[serde(rename = "value")]
    pub value: String,
}

// Online session

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenOnlineSessionRequest {
    #[serde(rename = "formCode")]
    pub form_code: FormCode,

    #[serde(rename = "encryption")]
    pub encryption: EncryptionInfo,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct OpenOnlineSessionResponse {
    #[serde(rename = "referenceNumber")]
    pub reference_number: String,

    #[serde(rename = "validUntil")]
    pub valid_until: DateTime<FixedOffset>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionStatusResponse {
    #[serde(rename = "status")]
    pub status: OperationStatusInfo,

    #[serde(rename = "invoiceCount")]
    pub invoice_count: Option<i32>,

    #[serde(rename = "successfulInvoiceCount")]
    pub successful_invoice_count: Option<i32>,

    #[serde(rename = "failedInvoiceCount")]
    pub failed_invoice_count: Option<i32>,

    #[serde(rename = "validUntil")]
    pub valid_until: Option<DateTime<FixedOffset>>,

    #[serde(rename = "dateCreated")]
    pub date_created: DateTime<FixedOffset>,

    #[serde(rename = "dateUpdated")]
    pub date_updated: DateTime<FixedOffset>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileMetadata {
    #[serde(rename = "hashSHA")]
    pub hash_sha: String,

    #[serde(rename = "fileSize")]
    pub file_size: usize,
}


// Batch session

#[derive(Debug)]
pub struct BatchPartSendingInfo {
    pub data: Vec<u8>,
    pub metadata: FileMetadata,
    pub ordinal_number: usize,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct OpenBatchSessionRequest {
    #[serde(rename = "formCode")]
    pub form_code: FormCode,

    #[serde(rename = "batchFile")]
    pub batch_file: BatchFileInfo,

    #[serde(rename = "encryption")]
    pub encryption: EncryptionInfo,

    #[serde(rename = "offlineMode")]
    pub offline_mode: bool,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct BatchFileInfo {
    #[serde(rename = "fileSize")]
    pub file_size: usize,

    #[serde(rename = "fileHash")]
    pub file_hash: String,

    #[serde(rename = "fileParts")]
    pub file_parts: Vec<BatchFilePartInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BatchFilePartInfo {
    #[serde(rename = "ordinalNumber")]
    pub ordinal_number: usize,

    #[serde(rename = "fileSize")]
    pub file_size: usize,

    #[serde(rename = "fileHash")]
    pub file_hash: String,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenBatchSessionResponse {
    #[serde(rename = "referenceNumber")]
    pub reference_number: String,

    #[serde(rename = "partUploadRequests")]
    pub part_upload_requests: Vec<PackagePartSignatureInitResponseType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackagePartSignatureInitResponseType {
    #[serde(rename = "method")]
    pub method: String,

    #[serde(rename = "ordinalNumber")]
    pub ordinal_number: usize,

    #[serde(rename = "url")]
    pub url: String,

    #[serde(rename = "headers")]
    pub headers: HashMap<String, String>,
}
