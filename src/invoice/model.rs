use chrono::{DateTime, offset::Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum SystemCode {
    #[serde(rename = "FA (2)")]
    FA2,

    #[serde(rename = "FA (3)")]
    FA3,

    #[serde(rename = "PEF (3)")]
    PEF,

    #[serde(rename = "PEF_KOR (3)")]
    PEFKOR,
}

impl SystemCode {
    pub fn system_code(self) -> &'static str {
        match self {
            SystemCode::FA2 => "FA (2)",
            SystemCode::FA3 => "FA (3)",
            SystemCode::PEF => "PEF (3)",
            SystemCode::PEFKOR => "PEF_KOR (3)",
        }
    }

    pub fn value(self) -> &'static str {
        match self {
            SystemCode::FA2 => "FA",
            SystemCode::FA3 => "FA",
            SystemCode::PEF => "PEF",
            SystemCode::PEFKOR => "PEF",
        }
    }

    pub fn schema_version(self) -> &'static str {
        match self {
            SystemCode::FA2 => "1-0E",
            SystemCode::FA3 => "1-0E",
            SystemCode::PEF => "2-1",
            SystemCode::PEFKOR => "2-1",
        }
    }
}

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
pub struct InvoiceStatusInfo {
    #[serde(rename = "code")]
    pub code: i32,

    #[serde(rename = "description")]
    pub description: String,

    #[serde(rename = "details")]
    pub details: Option<Vec<String>>,

    #[serde(rename = "extensions")]
    pub extensions: Option<HashMap<String, String>>,
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
