use chrono::{DateTime, Utc};
use ksef::error;

#[tokio::test]
async fn test_get_qrcode_nip_is_empty()  {
    let client = ksef::Client::new(ksef::Environment::Test, 2000);
    
    let nip = "".to_string();
    let issue_date: DateTime<Utc> = "2026-02-01 00:00:00 +00:00".parse().unwrap();  
    let invoice_hash = "xxx".to_string();

    let result = client.get_qrcode(&nip, &issue_date, &invoice_hash, Some(300)).await;

    assert!(matches!(
        result,
        Err(error::ErrorResponse { code, .. }) if code == "empty_value"
    ));
}

#[tokio::test]
async fn test_get_qrcode_invoice_hash_is_empty()  {
    let client = ksef::Client::new(ksef::Environment::Test, 2000);
    
    let nip = "xxx".to_string();
    let issue_date: DateTime<Utc> = "2026-02-01 00:00:00 +00:00".parse().unwrap();  
    let invoice_hash = "".to_string();

    let result = client.get_qrcode(&nip, &issue_date, &invoice_hash, Some(300)).await;

    assert!(matches!(
        result,
        Err(error::ErrorResponse { code, .. }) if code == "empty_value"
    ));
}

#[tokio::test]
async fn test_get_qrcode_build_url_error()  {
    let client = ksef::Client::new(ksef::Environment::Test, 2000);
    
    let nip = "xxx".to_string();
    let issue_date: DateTime<Utc> = "2026-02-01 00:00:00 +00:00".parse().unwrap();  
    let invoice_hash = "xxx".to_string();

    let result = client.get_qrcode(&nip, &issue_date, &invoice_hash, Some(300)).await;

    assert!(matches!(
        result,
        Err(error::ErrorResponse { code, .. }) if code == "build_url_error"
    ));
}
