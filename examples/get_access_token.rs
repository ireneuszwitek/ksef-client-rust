#[tokio::main]
async fn main() {
    
    let client = ksef::KsefClient::new(ksef::Environment::Prod, 2000).unwrap();

    let company_info = ksef::CompanyInfo {
        ksef_token: "<ksef_token>".to_string(),
        nip: "<nip>".to_string(),
    };

    let access_tokens = match client.get_access_tokens(&company_info).await {
        Ok(access_tokens) => access_tokens,
        Err(e) => {
            eprintln!("Error getting access_tokens: {}", e);
            return;
        }
    };
    
    let access_token = access_tokens.access_token.token;

    println!("access_token {}", access_token);
}
