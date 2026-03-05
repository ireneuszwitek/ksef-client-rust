use chrono::offset::Utc;

#[tokio::main]
async fn main() {

    let client = ksef::Client::new(ksef::Environment::Prod, 2000);
    let access_token = "<access_token>".to_string();

    let invoice_query_filters = ksef::invoice::InvoiceQueryFilters {
        subject_type: ksef::invoice::InvoiceSubjectType::Subject1,
        date_range: ksef::invoice::DateRange {
            from: Utc::now() - chrono::Duration::days(30),
            to: Some(Utc::now()),
            date_type: ksef::invoice::DateType::PermanentStorage,
            restrict_to_permanent_storage_hwm_date: Some(true),
        },
    };

    let export_invoice = match client.export_invoice(&invoice_query_filters, &access_token).await {
        Ok(export_invoice) => export_invoice,
        Err(e) => {
            eprintln!("Error: {}; {}", e.code, e.message);
            return;
        }
    };

    println!("{:#?}", export_invoice);
}