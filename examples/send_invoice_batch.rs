#[tokio::main]
async fn main() {

    let client = ksef::KsefClient::new(ksef::Environment::Prod, 2000).unwrap();
    let access_token = "<access_token>".to_string();

    let system_code = ksef::invoice::SystemCode::FA3;


    let mut list: Vec<(String, String)> = Vec::new();
    list.push(( "invoice1.xml".to_string(), std::fs::read_to_string("<path>/invoice1.xml").expect("Failed to read file") ));
    list.push(( "invoice2.xml".to_string(), std::fs::read_to_string("<path>/invoice2.xml").expect("Failed to read file") ));
    //println!("{:#?}", list);

    let send_invoice_batch_result = match client.send_invoice_batch(&access_token, &system_code, &list, 1).await {
        Ok(send_invoice_batch_result) => send_invoice_batch_result,
        Err(e) => {
            eprintln!("Error: {}; {}", e.code, e.message);
            return;
        }
    };

    
    println!("{:#?}", send_invoice_batch_result);
}
