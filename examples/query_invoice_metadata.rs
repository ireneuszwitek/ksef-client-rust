use chrono::offset::Utc;

#[tokio::main]
async fn main() {

    let client = ksef_client::KsefClient::new("https://api.ksef.mf.gov.pl".to_string(), 2000).unwrap();
    let access_token = "<access_token>".to_string();

    let invoice_query_filters = ksef_client::invoice::InvoiceQueryFilters {
        subject_type: ksef_client::invoice::InvoiceSubjectType::Subject1,
        date_range: ksef_client::invoice::DateRange {
            from: Utc::now() - chrono::Duration::days(90),
            to: Some(Utc::now()),
            date_type: ksef_client::invoice::DateType::Issue,
            restrict_to_permanent_storage_hwm_date: None,
        },
    };

    let query_invoice_metadata_result = match client.query_invoice_metadata(&invoice_query_filters, &access_token, 0, 10, ksef_client::invoice::SortOrder::Asc).await {
            Ok(query_invoice_metadata_result) => query_invoice_metadata_result,
            Err(e) => {
                eprintln!("Error: {}; {}", e.code, e.message);
                return;
            }
        };
        
    println!("{:#?}", query_invoice_metadata_result);    
}