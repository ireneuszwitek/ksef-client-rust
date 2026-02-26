use chrono::{DateTime, Utc};

#[tokio::main]
async fn main() {

    let client = ksef::KsefClient::new("https://api.ksef.mf.gov.pl".to_string(), 2000).unwrap();
    
    let qrcode_base_url = "https://qr.ksef.mf.gov.pl".to_string();
    let nip = "<nip>".to_string();
    let issue_date: DateTime<Utc> = "2026-02-01 00:00:00 +00:00".parse().unwrap();
    let invoice_hash = "<invoice_hash>".to_string();

    let qrcode = match client.get_qrcode(&qrcode_base_url, &nip, &issue_date, &invoice_hash, Some(300)).await {
        Ok(qrcode) => qrcode,
        Err(e) => {
            eprintln!("Error: {}; {}", e.code, e.message);
            return;
        }
    };

    std::fs::write("qr.png", qrcode).unwrap();
}