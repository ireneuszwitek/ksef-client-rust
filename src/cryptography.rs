use aes::Aes256;
use base64;
use base64::{Engine, engine::general_purpose::STANDARD};
use cbc::{Decryptor, Encryptor};
use cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit, block_padding::Pkcs7};
use pem_rfc7468::LineEnding;
use rand::{RngCore, rngs::OsRng};
use rsa::{
    RsaPublicKey,
    oaep::Oaep,
    pkcs8::{DecodePublicKey, EncodePublicKey},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::cell::RefCell;
use x509_parser::{parse_x509_certificate, pem::parse_x509_pem};

use crate::{certificates, common};

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
pub struct FileMetadata {
    #[serde(rename = "hashSHA")]
    pub hash_sha: String,

    #[serde(rename = "fileSize")]
    pub file_size: usize,
}

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

pub fn encrypt_bytes_with_aes256(content: &[u8], key: &[u8], iv: &[u8]) -> Vec<u8> {
    let buf = content.to_vec();

    Encryptor::<Aes256>::new_from_slices(key, iv)
        .unwrap()
        .encrypt_padded_vec_mut::<Pkcs7>(&buf)
}

pub fn decrypt_bytes_with_aes256(content: &[u8], key: &[u8], iv: &[u8]) -> Result<Vec<u8>, String> {
    let decryptor = Decryptor::<Aes256>::new_from_slices(key, iv)
        .map_err(|e| format!("Invalid key/iv: {e}"))?;

    decryptor
        .decrypt_padded_vec_mut::<Pkcs7>(content)
        .map_err(|e| format!("Decryption failed: {e}"))
}

pub(crate) async fn get_encryption_data(
    base_url: &common::Url,
    public_certificates: &RefCell<Option<Vec<certificates::PemCertificateInfo>>>,
) -> Result<EncryptionData, &'static str> {
    let key = generate_random_256_bits_key();
    let iv = generate_random_16_bytes_iv();

    let symetric_cert = match certificates::public_certificate(
        &base_url,
        &public_certificates,
        &certificates::PublicKeyCertificateUsage::SymmetricKeyEncryption,
    )
    .await
    {
        Ok(symetric_cert) => symetric_cert,
        Err(_) => {
            return Err("get symetric_cert error");
        }
    };

    let encrypted_key: Vec<u8> =
        encrypt_ksef_token_with_rsa_using_public_key(&symetric_cert, &key).unwrap();

    let encryption_info = EncryptionInfo {
        encrypted_symmetric_key: STANDARD.encode(&encrypted_key),
        initialization_vector: STANDARD.encode(&iv),
    };

    let encrypted_data = EncryptionData {
        cipher_key: key,
        cipher_iv: iv,
        encryption_info,
    };

    Ok(encrypted_data)
}

pub(crate) fn get_metadata(file: &[u8]) -> FileMetadata {
    let mut hasher = Sha256::new();
    hasher.update(file);
    let hash = hasher.finalize();

    FileMetadata {
        file_size: file.len(),
        hash_sha: STANDARD.encode(hash),
    }
}
