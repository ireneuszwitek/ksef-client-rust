use chrono::{DateTime, Utc};

#[tokio::main]
async fn main() {

    let client = ksef::Client::new(ksef::Environment::Prod, 2000);
    
    let nip = "<nip>".to_string();
    let issue_date: DateTime<Utc> = "RRRR-MM-DD HH:MM:SS +00:00".parse().unwrap();  
    
    let invoice_hash = "<invoice_hash>".to_string();

    let qrcode = match client.get_qrcode(&nip, &issue_date, &invoice_hash, Some(300)).await {
        Ok(qrcode) => qrcode,
        Err(e) => {
            eprintln!("Error: {}; {}", e.code, e.message);
            return;
        }
    };

    std::fs::write("qr.png", qrcode).unwrap();
}