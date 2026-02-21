use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use base64;
use rand::RngCore;
use rand::rngs::OsRng;
use rsa::oaep::Oaep;
use rsa::{
    RsaPublicKey,
    pkcs8::{DecodePublicKey, EncodePublicKey},
};
use sha2::Sha256;

use pem_rfc7468::LineEnding;
use x509_parser::parse_x509_certificate;
use x509_parser::pem::parse_x509_pem;


use aes::Aes256;
use block_modes::block_padding::Pkcs7;
use block_modes::{BlockMode, Cbc};

type Aes256Cbc = Cbc<Aes256, Pkcs7>;

use crate::{KsefClient, models, certificates};

pub(crate) fn export_public_key_to_pem(
    rsa: &RsaPublicKey,
) -> Result<String, Box<dyn std::error::Error>> {
    Ok(rsa.to_public_key_pem(LineEnding::LF)?)
}

pub(crate) fn get_rsa_public_pem(
    certificate_pem: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let (_, pem) = parse_x509_pem(certificate_pem.as_bytes())?;
    let (_, cert) = parse_x509_certificate(&pem.contents)?;

    let spki_der = cert.public_key().raw;

    let rsa_pub = RsaPublicKey::from_public_key_der(spki_der)?;
    let pub_pem = export_public_key_to_pem(&rsa_pub)?;
    Ok(pub_pem)
}

pub(crate) fn encrypt_ksef_token_with_rsa_using_public_key(
    ksef_token_cert: &String,
    content: &[u8],
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let public_key_pem = get_rsa_public_pem(ksef_token_cert)?;
    let public_key = RsaPublicKey::from_public_key_pem(public_key_pem.as_str())?;
    let mut rng = OsRng;
    let encrypted = public_key.encrypt(&mut rng, Oaep::new::<Sha256>(), &content)?;

    Ok(encrypted)
}

pub(crate) fn generate_random_256_bits_key() -> Vec<u8> {
    let mut key = vec![0u8; 32]; // 256 / 8
    OsRng.fill_bytes(&mut key);
    key
}

pub(crate) fn generate_random_16_bytes_iv() -> Vec<u8> {
    let mut iv = vec![0u8; 16];
    OsRng.fill_bytes(&mut iv);
    iv
}

pub(crate) fn decrypt_bytes_with_aes256(
    content: &[u8],
    key: &[u8],
    iv: &[u8],
) -> Result<Vec<u8>, String> {
    if key.len() != 32 {
        return Err("AES-256 key must be 32 bytes".into());
    }
    if iv.len() != 16 {
        return Err("AES-CBC IV must be 16 bytes".into());
    }

    let cipher =
        Aes256Cbc::new_from_slices(key, iv).map_err(|e| format!("Cipher init error: {e}"))?;

    cipher
        .decrypt_vec(content)
        .map_err(|e| format!("Decrypt error: {e}"))
}

    pub(crate) async fn get_encryption_data(client: &KsefClient) -> Result<models::EncryptionData, &str> {
        let key = generate_random_256_bits_key();
        let iv = generate_random_16_bytes_iv();

        let symetric_cert = match certificates::
            public_certificate(&client, &models::PublicKeyCertificateUsage::SymmetricKeyEncryption)
            .await
        {
            Ok(symetric_cert) => symetric_cert,
            Err(_) => {
                return Err("get symetric_cert error");
            }
        };

        let encrypted_key: Vec<u8> = encrypt_ksef_token_with_rsa_using_public_key(&symetric_cert, &key).unwrap();

        let encryption_info = models::EncryptionInfo {
            encrypted_symmetric_key: STANDARD.encode(&encrypted_key),
            initialization_vector: STANDARD.encode(&iv),
        };

        let encrypted_data = models::EncryptionData {
            cipher_key: key,
            cipher_iv: iv,
            encryption_info,
        };

        Ok(encrypted_data)
    }
