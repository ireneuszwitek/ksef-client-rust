use ksef::error;

#[tokio::test]
async fn test_get_sessions_access_token_is_empty()  {
    let client = ksef::Client::new(ksef::Environment::Test, 2000);
    let access_token = "".to_string();

    let result = client.get_sessions(ksef::session::SessionType::Batch, &access_token, 10, &None).await;

    assert!(matches!(
        result,
        Err(error::ErrorResponse { code, .. }) if code == "empty_value"
    ));
}
