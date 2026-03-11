use chrono::Utc;
use ksef::error;

#[tokio::test]
async fn test_export_invoice_access_token_is_empty()  {
    let client = ksef::Client::new(ksef::Environment::Test, 2000);
    let access_token = "".to_string();

    let invoice_query_filters = ksef::invoice::InvoiceQueryFilters {
        subject_type: ksef::invoice::InvoiceSubjectType::Subject1,
        date_range: ksef::invoice::DateRange {
            from: Utc::now() - chrono::Duration::days(30),
            to: Some(Utc::now()),
            date_type: ksef::invoice::DateType::PermanentStorage,
            restrict_to_permanent_storage_hwm_date: Some(true),
        },
    };

    let result = client.export_invoice(&invoice_query_filters, &access_token).await;

    assert!(matches!(
        result,
        Err(error::ErrorResponse { code, .. }) if code == "empty_value"
    ));
}
