use chrono::offset::Utc;

#[tokio::main]
async fn main() {

    let client = ksef::KsefClient::new("https://api.ksef.mf.gov.pl".to_string(), 2000).unwrap();

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

    // if access token has expired
    if access_tokens.access_token.valid_until < Utc::now() {
        // refresh token ->
        let access_token_info = match client
            .refresh_access_token(&access_tokens.refresh_token.token)
            .await
        {
            Ok(access_token_info) => access_token_info,
            Err(e) => {
                eprintln!("Error getting refresh token: {}", e);
                return;
            }
        };

        let access_token = access_token_info.token;

        println!("access_token {}", access_token);
    }
}
