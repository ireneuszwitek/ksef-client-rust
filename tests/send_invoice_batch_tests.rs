use ksef::error;

#[tokio::test]
async fn test_send_invoice_batch_access_token_is_empty()  {
    let client = ksef::Client::new(ksef::Environment::Test, 2000);
    let access_token = "".to_string();
    let system_code = ksef::invoice::SystemCode::FA3;
    let list: Vec<(String, String)> = vec![("invoice1.xm".to_string(), "<path>/invoice1.xml".to_string())];

    let result = client.send_invoice_batch(&access_token, &system_code, &list, 1).await;

    assert!(matches!(
        result,
        Err(error::ErrorResponse { code, .. }) if code == "empty_value"
    ));
}

#[tokio::test]
async fn test_send_invoice_batch_list_is_empty()  {
    let client = ksef::Client::new(ksef::Environment::Test, 2000);
    let access_token = "xxx".to_string();
    let system_code = ksef::invoice::SystemCode::FA3;
    let list: Vec<(String, String)> = Vec::new();

    let result = client.send_invoice_batch(&access_token, &system_code, &list, 1).await;

    assert!(matches!(
        result,
        Err(error::ErrorResponse { code, .. }) if code == "empty_list"
    ));
}

#[tokio::test]
async fn test_get_session_upo_session_reference_number_is_empty()  {
    let client = ksef::Client::new(ksef::Environment::Test, 2000);
    let session_reference_number = "".to_string();
    let upo_reference_number = "xxx".to_string();
    let access_token = "xxx".to_string();

    let result = client.get_session_upo(&session_reference_number, &upo_reference_number, &access_token).await;

    assert!(matches!(
        result,
        Err(error::ErrorResponse { code, .. }) if code == "empty_value"
    ));
}

#[tokio::test]
async fn test_get_session_upo_upo_reference_number_is_empty()  {
    let client = ksef::Client::new(ksef::Environment::Test, 2000);
    let session_reference_number = "xxx".to_string();
    let upo_reference_number = "".to_string();
    let access_token = "xxx".to_string();

    let result = client.get_session_upo(&session_reference_number, &upo_reference_number, &access_token).await;

    assert!(matches!(
        result,
        Err(error::ErrorResponse { code, .. }) if code == "empty_value"
    ));
}

#[tokio::test]
async fn test_get_session_upo_access_token_is_empty()  {
    let client = ksef::Client::new(ksef::Environment::Test, 2000);
    let session_reference_number = "xxx".to_string();
    let upo_reference_number = "xxx".to_string();
    let access_token = "".to_string();

    let result = client.get_session_upo(&session_reference_number, &upo_reference_number, &access_token).await;

    assert!(matches!(
        result,
        Err(error::ErrorResponse { code, .. }) if code == "empty_value"
    ));
}

#[tokio::test]
async fn test_get_session_invoice_session_reference_number_is_empty()  {
    let client = ksef::Client::new(ksef::Environment::Test, 2000);
    let session_reference_number = "".to_string();
    let access_token = "xxx".to_string();

    let result = client.get_session_invoice(&session_reference_number, &access_token, 10).await;

    assert!(matches!(
        result,
        Err(error::ErrorResponse { code, .. }) if code == "empty_value"
    ));
}

#[tokio::test]
async fn test_get_session_invoice_session_access_token_is_empty()  {
    let client = ksef::Client::new(ksef::Environment::Test, 2000);
    let session_reference_number = "xxx".to_string();
    let access_token = "".to_string();

    let result = client.get_session_invoice(&session_reference_number, &access_token, 10).await;

    assert!(matches!(
        result,
        Err(error::ErrorResponse { code, .. }) if code == "empty_value"
    ));
}

#[tokio::test]
async fn test_get_upo_download_url_is_empty()  {
    let client = ksef::Client::new(ksef::Environment::Test, 2000);
    let upo_download_url = "".to_string();

    let result = client.get_upo(&upo_download_url).await;

    assert!(matches!(
        result,
        Err(error::ErrorResponse { code, .. }) if code == "empty_value"
    ));
}