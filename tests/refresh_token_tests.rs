use ksef::error;

#[tokio::test]
async fn test_refresh_access_token_refresh_token_is_empty()  {
    let client = ksef::Client::new(ksef::Environment::Test, 2000);
    let refresh_token = "".to_string();

    let result = client.refresh_access_token(&refresh_token).await;

    assert!(matches!(
        result,
        Err(error::ErrorResponse { code, .. }) if code == "empty_value"
    ));
}
