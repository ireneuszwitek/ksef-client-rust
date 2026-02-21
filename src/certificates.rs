use crate::{KsefClient, models, certificates};

pub(crate) async fn get_public_certificates(
    base_url: &String,
) -> Result<Vec<models::PemCertificateInfo>, reqwest::Error> {
    let url = "/v2/security/public-key-certificates";

    let reqwest_client = reqwest::Client::new();
    let result = reqwest_client
        .get(format!("{}{}", base_url, url))
        .send()
        .await?
        .json::<Vec<models::PemCertificateInfo>>()
        .await?;
    Ok(result)
}

pub(crate) async fn public_certificate(
        client: &KsefClient,
        certificate_usage: &models::PublicKeyCertificateUsage,
    ) -> Result<String, &'static str> {
        // Checking if public certificates have already been downloaded
        if client.public_certificates.borrow().is_none() {
            *client.public_certificates.borrow_mut() = match certificates::get_public_certificates(&client.base_url).await {
                Ok(public_certificates) => Some(public_certificates),
                Err(_) => {
                    return Err("get public certificates error");
                }
            };
        }

        if let Some(public_certificates) = &*client.public_certificates.borrow() {
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
