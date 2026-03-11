use ksef::error;

#[tokio::test]
async fn test_open_online_session_access_token_is_empty()  {
    let client = ksef::Client::new(ksef::Environment::Test, 2000);
    let access_token = "".to_string();
    let system_code = ksef::invoice::SystemCode::FA3;

    // Generate encryption
    let encryption = match client.get_encryption_data().await {
        Ok(encryption) => encryption,
        Err(e) => {
            eprintln!("Error encryption: {}", e);
            return;
        }
    };

    let result = client.open_online_session(&encryption, &access_token, &system_code).await;

    assert!(matches!(
        result,
        Err(error::ErrorResponse { code, .. }) if code == "empty_value"
    ));
}

#[tokio::test]
async fn test_send_invoice_reference_number_is_empty()  {
    let client = ksef::Client::new(ksef::Environment::Test, 2000);
    let reference_number = "".to_string();
    let access_token = "xxx".to_string();
    let xml = "<file_content>".to_string();

    // Generate encryption
    let encryption = match client.get_encryption_data().await {
        Ok(encryption) => encryption,
        Err(e) => {
            eprintln!("Error encryption: {}", e);
            return;
        }
    };

    let result = client.send_invoice(&reference_number, &access_token, &encryption, &xml).await;

    assert!(matches!(
        result,
        Err(error::ErrorResponse { code, .. }) if code == "empty_value"
    ));
}

#[tokio::test]
async fn test_send_invoice_access_token_is_empty()  {
    let client = ksef::Client::new(ksef::Environment::Test, 2000);
    let reference_number = "xxx".to_string();
    let access_token = "".to_string();
    let xml = "<file_content>".to_string();

    // Generate encryption
    let encryption = match client.get_encryption_data().await {
        Ok(encryption) => encryption,
        Err(e) => {
            eprintln!("Error encryption: {}", e);
            return;
        }
    };

    let result = client.send_invoice(&reference_number, &access_token, &encryption, &xml).await;

    assert!(matches!(
        result,
        Err(error::ErrorResponse { code, .. }) if code == "empty_value"
    ));
}

#[tokio::test]
async fn test_send_invoice_xml_is_empty()  {
    let client = ksef::Client::new(ksef::Environment::Test, 2000);
    let reference_number = "xxx".to_string();
    let access_token = "xxx".to_string();
    let xml = "".to_string();

    // Generate encryption
    let encryption = match client.get_encryption_data().await {
        Ok(encryption) => encryption,
        Err(e) => {
            eprintln!("Error encryption: {}", e);
            return;
        }
    };

    let result = client.send_invoice(&reference_number, &access_token, &encryption, &xml).await;

    assert!(matches!(
        result,
        Err(error::ErrorResponse { code, .. }) if code == "empty_value"
    ));
}

#[tokio::test]
async fn test_get_online_session_status_reference_number_is_empty()  {
    let client = ksef::Client::new(ksef::Environment::Test, 2000);
    let reference_number = "".to_string();
    let access_token = "xxx".to_string();

    let result = client.get_online_session_status(&reference_number, &access_token).await;

    assert!(matches!(
        result,
        Err(error::ErrorResponse { code, .. }) if code == "empty_value"
    ));
}

#[tokio::test]
async fn test_get_online_session_status_access_token_is_empty()  {
    let client = ksef::Client::new(ksef::Environment::Test, 2000);
    let reference_number = "xxx".to_string();
    let access_token = "".to_string();

    let result = client.get_online_session_status(&reference_number, &access_token).await;

    assert!(matches!(
        result,
        Err(error::ErrorResponse { code, .. }) if code == "empty_value"
    ));
}

#[tokio::test]
async fn test_close_online_session_reference_number_is_empty()  {
    let client = ksef::Client::new(ksef::Environment::Test, 2000);
    let reference_number = "".to_string();
    let access_token = "xxx".to_string();

    let result = client.close_online_session(&reference_number, &access_token).await;

    assert!(matches!(
        result,
        Err(error::ErrorResponse { code, .. }) if code == "empty_value"
    ));
}

#[tokio::test]
async fn test_close_online_session_access_token_is_empty()  {
    let client = ksef::Client::new(ksef::Environment::Test, 2000);
    let reference_number = "xxx".to_string();
    let access_token = "".to_string();

    let result = client.close_online_session(&reference_number, &access_token).await;

    assert!(matches!(
        result,
        Err(error::ErrorResponse { code, .. }) if code == "empty_value"
    ));
}
