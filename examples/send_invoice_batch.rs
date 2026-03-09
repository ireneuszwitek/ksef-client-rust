#[tokio::main]
async fn main() {
    let client = ksef::Client::new(ksef::Environment::Prod, 2000);
    let access_token = "<access_token>".to_string();

    let system_code = ksef::invoice::SystemCode::FA3;

    let mut list: Vec<(String, String)> = Vec::new();
    list.push((
        "invoice1.xml".to_string(),
        std::fs::read_to_string("<path>/invoice1.xml").expect("Failed to read file"),
    ));
    list.push((
        "invoice2.xml".to_string(),
        std::fs::read_to_string("<path>/invoice2.xml").expect("Failed to read file"),
    ));
    //println!("{:#?}", list);


    // sending invoices in batch mode
    let (session_reference_number, send_invoice_batch_result) = match client
        .send_invoice_batch(&access_token, &system_code, &list, 1)
        .await
    {
        Ok(send_invoice_batch_result) => send_invoice_batch_result,
        Err(e) => {
            eprintln!("Error: {}; {}", e.code, e.message);
            return;
        }
    };

    // get collective session UPO
    if let Some(upo_page) = send_invoice_batch_result
        .upo
        .as_ref()
        .and_then(|u| u.pages.first())
    {
        let upo = match client
            .get_session_upo(
                &session_reference_number,
                &upo_page.reference_number,
                &access_token,
            )
            .await
        {
            Ok(upo) => upo,
            Err(e) => {
                eprintln!("Error: {}; {}", e.code, e.message);
                return;
            }
        };

        println!("upo: {:#?}", upo);
    }

    // get session invoices
    let session_invoice_response = match client
        .get_session_invoice(&session_reference_number, &access_token, 10)
        .await
    {
        Ok(session_invoice_response) => session_invoice_response,
        Err(e) => {
            eprintln!("Error: {}; {}", e.code, e.message);
            return;
        }
    };


    // get UPO for first incoice
    if let Some(invoice) = session_invoice_response.invoices.first() {
        if let Some(upo_download_url) = &invoice.upo_download_url {
            let upo = match client.get_upo(&upo_download_url).await {
                Ok(upo) => upo,
                Err(e) => {
                    eprintln!("Error: {}; {}", e.code, e.message);
                    return;
                }
            };

            println!(
                "UPO for invoice\n ksef_number: {};\n upo: {:#?}",
                invoice.ksef_number.as_deref().unwrap_or(""),
                upo
            );
        }
    }
}
