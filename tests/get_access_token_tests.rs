use ksef::error;

#[tokio::test]
async fn test_get_access_tokens_ksef_token_is_empty()  {
    let client = ksef::Client::new(ksef::Environment::Test, 2000);
    let company_info = ksef::CompanyInfo {
        ksef_token: "".to_string(),
        nip: "xxx".to_string(),
    };

    let result = client.get_access_tokens(&company_info).await;

    assert!(matches!(
        result,
        Err(error::ErrorResponse { code, .. }) if code == "empty_value"
    ));
}

#[tokio::test]
async fn test_get_access_tokens_nip_is_empty()  {
    let client = ksef::Client::new(ksef::Environment::Test, 2000);
    let company_info = ksef::CompanyInfo {
        ksef_token: "xxx".to_string(),
        nip: "".to_string(),
    };

    let result = client.get_access_tokens(&company_info).await;

    assert!(matches!(
        result,
        Err(error::ErrorResponse { code, .. }) if code == "empty_value"
    ));
}
