use crate::common;
use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct PemCertificateInfo {
    #[serde(rename = "certificate")]
    pub(crate) certificate: String,

    #[serde(rename = "validFrom")]
    pub(crate) valid_from: DateTime<FixedOffset>,

    #[serde(rename = "validTo")]
    pub(crate) valid_to: DateTime<FixedOffset>,

    #[serde(rename = "usage")]
    pub(crate) usage: Vec<PublicKeyCertificateUsage>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) enum PublicKeyCertificateUsage {
    #[serde(rename = "KsefTokenEncryption")]
    KsefTokenEncryption,

    #[serde(rename = "SymmetricKeyEncryption")]
    SymmetricKeyEncryption,
}

async fn get_public_certificates(
    base_url: &common::Url,
) -> Result<Vec<PemCertificateInfo>, reqwest::Error> {
    let url = "/v2/security/public-key-certificates";

    let reqwest_client = reqwest::Client::new();
    let result = reqwest_client
        .get(base_url.join(url))
        .send()
        .await?
        .json::<Vec<PemCertificateInfo>>()
        .await?;
    Ok(result)
}

pub(crate) async fn public_certificate(
    base_url: &common::Url,
    public_certificates: &RefCell<Option<Vec<PemCertificateInfo>>>,
    certificate_usage: &PublicKeyCertificateUsage,
) -> Result<String, &'static str> {
    // Checking if public certificates have already been downloaded
    if public_certificates.borrow().is_none() {
        *public_certificates.borrow_mut() = match get_public_certificates(&base_url).await {
            Ok(public_certificates) => Some(public_certificates),
            Err(_) => {
                return Err("get public certificates error");
            }
        };
    }

    if let Some(public_certificates) = &*public_certificates.borrow() {
        let cert = match public_certificates
            .iter()
            .find(|info| info.usage.contains(&certificate_usage))
            .map(|info| &info.certificate)
        {
            Some(cert) => format!(
                "-----BEGIN CERTIFICATE-----\n{}\n-----END CERTIFICATE-----",
                cert
            ),
            None => {
                return Err("get public certificate error");
            }
        };
        return Ok(cert);
    }

    Err("unknown error")
}
