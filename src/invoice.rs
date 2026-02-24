use serde::{Deserialize, Serialize};
use chrono::{DateTime, offset::Utc};
use std::fmt;
use std::collections::HashMap;
use crate::models;

/////////////////////////////////
///  Invoice query
/////////////////////////////////

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum InvoiceSubjectType {
    #[serde(rename = "Subject1")]
    Subject1,
    #[serde(rename = "Subject2")]
    Subject2,
    #[serde(rename = "Subject3")]
    Subject3,
    #[serde(rename = "SubjectAuthorized")]
    SubjectAuthorized,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DateRange {
    #[serde(rename = "DateType")]
    pub date_type: DateType,

    #[serde(rename = "From")]
    pub from: DateTime<Utc>,

    #[serde(rename = "To")]
    pub to: Option<DateTime<Utc>>,

    #[serde(rename = "RestrictToPermanentStorageHwmDate")]
    pub restrict_to_permanent_storage_hwm_date: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum DateType {
    #[serde(rename = "Issue")]
    Issue,

    #[serde(rename = "Invoicing")]
    Invoicing,

    #[serde(rename = "PermanentStorage")]
    PermanentStorage,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InvoiceQueryFilters {
    #[serde(rename = "SubjectType")]
    pub subject_type: InvoiceSubjectType,

    #[serde(rename = "DateRange")]
    pub date_range: DateRange,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SortOrder {
    #[serde(rename = "Asc")]
    Asc,
    #[serde(rename = "Desc")]
    Desc,
}
impl fmt::Display for SortOrder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SortOrder::Asc => write!(f, "Asc"),
            SortOrder::Desc => write!(f, "Desc"),
        }
    }
}

/////////////////////////////////
///  Invoice response
/////////////////////////////////

#[derive(Debug, Serialize, Deserialize)]
pub struct PagedInvoiceResponse {
    #[serde(rename = "hasMore")]
    pub has_more: bool,

    #[serde(rename = "isTruncated")]
    pub is_truncated: bool,

    #[serde(rename = "invoices")]
    pub invoices: Vec<InvoiceSummary>,

    #[serde(rename = "permanentStorageHwmDate")]
    pub permanent_storage_hwm_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InvoiceSummary {
    #[serde(rename = "ksefNumber")]
    pub ksef_number: String,

    #[serde(rename = "invoiceNumber")]
    pub invoice_number: String,

    #[serde(rename = "issueDate")]
    pub issue_date: String,

    #[serde(rename = "invoicingDate")]
    pub invoicing_date: DateTime<Utc>,

    #[serde(rename = "acquisitionDate")]
    pub acquisition_date: DateTime<Utc>,

    #[serde(rename = "permanentStorageDate")]
    pub permanent_storage_date: DateTime<Utc>,

    #[serde(rename = "seller")]
    pub seller: Seller,

    #[serde(rename = "buyer")]
    pub buyer: Buyer,

    #[serde(rename = "netAmount")]
    pub net_amount: f64,

    #[serde(rename = "grossAmount")]
    pub gross_amount: f64,

    #[serde(rename = "vatAmount")]
    pub vat_amount: f64,

    #[serde(rename = "currency")]
    pub currency: String,

    #[serde(rename = "invoicingMode")]
    pub invoicing_mode: InvoicingMode,

    #[serde(rename = "invoiceType")]
    pub invoice_type: InvoiceType,

    #[serde(rename = "formCode")]
    pub form_code: FormCode,

    #[serde(rename = "isSelfInvoicing")]
    pub is_self_invoicing: bool,

    #[serde(rename = "hasAttachment")]
    pub has_attachment: bool,

    #[serde(rename = "invoiceHash")]
    pub invoice_hash: String,

    #[serde(rename = "hashOfCorrectedInvoice")]
    pub hash_of_corrected_invoice: Option<String>,

    #[serde(rename = "thirdSubjects")]
    pub third_subjects: Option<Vec<ThirdSubject>>,

    #[serde(rename = "authorizedSubject")]
    pub authorized_subject: Option<AuthorizedSubject>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Seller {
    #[serde(rename = "nip")]
    pub nip: String,

    #[serde(rename = "name")]
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Buyer {
    #[serde(rename = "identifier")]
    pub identifier: BuyerIdentifier,

    #[serde(rename = "name")]
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BuyerIdentifier {
    #[serde(rename = "type")]
    pub kind: BuyerIdentifierType,

    #[serde(rename = "value")]
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum BuyerIdentifierType {
    #[serde(rename = "None")]
    None,
    #[serde(rename = "Other")]
    Other,
    #[serde(rename = "Nip")]
    Nip,
    #[serde(rename = "VatUe")]
    VatUe,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum InvoicingMode {
    #[serde(rename = "Online")]
    Online,
    #[serde(rename = "Offline")]
    Offline,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum InvoiceType {
    #[serde(rename = "Vat")]
    Vat,
    #[serde(rename = "Zal")]
    Zal,
    #[serde(rename = "Kor")]
    Kor,
    #[serde(rename = "Roz")]
    Roz,
    #[serde(rename = "Upr")]
    Upr,
    #[serde(rename = "KorZal")]
    KorZal,
    #[serde(rename = "KorRoz")]
    KorRoz,
    #[serde(rename = "VatPef")]
    VatPef,
    #[serde(rename = "VatPefSp")]
    VatPefSp,
    #[serde(rename = "KorPef")]
    KorPef,
    #[serde(rename = "VatRr")]
    VatRr,
    #[serde(rename = "KorVatRr")]
    KorVatRr,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FormCode {
    #[serde(rename = "systemCode")]
    pub system_code: String,

    #[serde(rename = "schemaVersion")]
    pub schema_version: String,

    #[serde(rename = "value")]
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ThirdSubject {
    #[serde(rename = "identifier")]
    pub identifier: ThirdSubjectIdentifier,

    #[serde(rename = "name")]
    pub name: String,

    #[serde(rename = "role")]
    pub role: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ThirdSubjectIdentifier {
    #[serde(rename = "type")]
    pub kind: ThirdSubjectIdentifierType,

    #[serde(rename = "value")]
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ThirdSubjectIdentifierType {
    #[serde(rename = "None")]
    None,
    #[serde(rename = "Other")]
    Other,
    #[serde(rename = "Nip")]
    Nip,
    #[serde(rename = "VatUe")]
    VatUe,
    #[serde(rename = "InternalId")]
    InternalId,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthorizedSubject {
    #[serde(rename = "nip")]
    pub nip: String,

    #[serde(rename = "name")]
    pub name: String,

    #[serde(rename = "role")]
    pub role: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InvoiceExportRequest {
    /// Informacje o szyfrowaniu paczki (RSA OAEP-SHA256, Base64).
    #[serde(rename = "Encryption")]
    pub encryption: models::EncryptionInfo,

    /// Filtry wyboru faktur do eksportu.
    #[serde(rename = "Filters")]
    pub filters: InvoiceQueryFilters,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OperationResponse {
    #[serde(rename = "referenceNumber")]
    pub reference_number: String,
}

// Export status response

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct InvoiceExportStatusResponse {
    #[serde(rename = "status")]
    pub(crate) status: models::OperationStatusInfo,

    #[serde(rename = "completedDate")]
    pub(crate) completed_date: Option<DateTime<Utc>>,

    #[serde(rename = "packageExpirationDate")]
    pub(crate) package_expiration_date: Option<DateTime<Utc>>,

    #[serde(rename = "package")]
    pub(crate) package: InvoiceExportPackage,
}

impl models::HasOperationStatusInfo for InvoiceExportStatusResponse { 
    fn status(&self) -> &models::OperationStatusInfo { 
        &self.status 
    } 
}


#[derive(Debug, Serialize, Deserialize)]
pub struct InvoiceExportPackage {
    #[serde(rename = "invoiceCount")]
    pub invoice_count: i32,

    #[serde(rename = "size")]
    pub size: i64,

    #[serde(rename = "parts")]
    pub parts: Vec<InvoiceExportPackagePart>,

    #[serde(rename = "isTruncated")]
    pub is_truncated: bool,

    #[serde(rename = "lastIssueDate")]
    pub last_issue_date: Option<DateTime<Utc>>,

    #[serde(rename = "lastInvoicingDate")]
    pub last_invoicing_date: Option<DateTime<Utc>>,

    #[serde(rename = "lastPermanentStorageDate")]
    pub last_permanent_storage_date: Option<DateTime<Utc>>,

    #[serde(rename = "permanentStorageHwmDate")]
    pub permanent_storage_hwm_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InvoiceExportPackagePart {
    #[serde(rename = "ordinalNumber")]
    pub ordinal_number: i32,

    #[serde(rename = "partName")]
    pub part_name: String,

    #[serde(rename = "method")]
    pub method: String,

    #[serde(rename = "url")]
    pub url: String,

    #[serde(rename = "partSize")]
    pub part_size: i64,

    #[serde(rename = "partHash")]
    pub part_hash: String,

    #[serde(rename = "encryptedPartSize")]
    pub encrypted_part_size: i64,

    #[serde(rename = "encryptedPartHash")]
    pub encrypted_part_hash: String,

    #[serde(rename = "expirationDate")]
    pub expiration_date: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InvoicePackageMetadata {
    pub invoices: Option<Vec<InvoiceSummary>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InvoiceExportResult {
    pub metadata_summaries: Vec<InvoiceSummary>,
    pub xml_files: HashMap<String, String>,
    pub is_truncated: bool,
    pub last_permanent_storage_date: Option<DateTime<Utc>>,
    pub permanent_storage_hwm_date: Option<DateTime<Utc>>,
}