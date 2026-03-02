#[tokio::main]
async fn main() {

    let client = ksef::KsefClient::new(ksef::Environment::Prod, 2000).unwrap();
    let access_token = "<access_token>".to_string();

    let system_code = ksef::invoice::SystemCode::FA3;

    // Generate encryption
    let encryption = match client.get_encryption_data().await {
        Ok(encryption) => encryption,
        Err(e) => {
            eprintln!("Error encryption: {}", e);
            return;
        }
    };

    // Open online session
    let open_online_session_result = match client
        .open_online_session(&encryption, &access_token, &system_code)
        .await
    {
        Ok(open_online_session_result) => open_online_session_result,
        Err(e) => {
            eprintln!("Error open_online_session_result: {}; {}", e.code, e.message);
            return;
        }
    };

    println!("{:#?}", open_online_session_result);

    // Load invoice xml
    let xml = std::fs::read_to_string("<path>/invoice.xml").expect("Failed to read file");

    println!("{}", xml);

    let send_invoice_result = match client
        .send_invoice(&open_online_session_result.reference_number, &access_token, &encryption, &xml)
        .await
    {
        Ok(send_invoice_result) => send_invoice_result,
        Err(e) => {
            eprintln!("Error send_invoice_result: {}; {}", e.code, e.message);
            return;
        }
    };

    println!("{:#?}", send_invoice_result);

    // Wait for the session to process all invoices
    let online_session_status = match client
        .get_online_session_status(&open_online_session_result.reference_number, &access_token)
        .await
    {
        Ok(online_session_status) => online_session_status,
        Err(e) => {
            eprintln!("Error online_session_status: {}; {}", e.code, e.message);
            return;
        }
    };

    println!("online_session_status: {:#?}", online_session_status);

    // Close online session
    if let Err(e) = client.close_online_session(&open_online_session_result.reference_number, &access_token).await{
        eprintln!("Error close_online_session: {}; {}", e.code, e.message);
        return;
    }

}
