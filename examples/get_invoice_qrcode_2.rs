use chrono::Utc;

#[tokio::main]
async fn main() {
    let client = ksef::Client::new(ksef::Environment::Prod, 2000);

    let company_info = ksef::CompanyInfo {
        ksef_token: "<ksef_token>".to_string(),
        nip: "<nip>".to_string(),
    };

    // get access token
    let access_tokens = match client.get_access_tokens(&company_info).await {
        Ok(access_tokens) => access_tokens,
        Err(e) => {
            eprintln!("Error: {}; {}", e.code, e.message);
            return;
        }
    };
    let access_token = access_tokens.access_token.token;

    // invoice selection filter
    let invoice_query_filters = ksef::invoice::InvoiceQueryFilters {
        subject_type: ksef::invoice::InvoiceSubjectType::Subject1,
        date_range: ksef::invoice::DateRange {
            from: Utc::now() - chrono::Duration::days(30),
            to: Some(Utc::now()),
            date_type: ksef::invoice::DateType::Issue,
            restrict_to_permanent_storage_hwm_date: None,
        },
    };
    // println!("{:#?}", invoice_query_filters);

    // get invoices
    let query_invoice_metadata_result = match client
        .query_invoice_metadata(
            &invoice_query_filters,
            &access_token,
            0,
            10,
            ksef::invoice::SortOrder::Asc,
        )
        .await
    {
        Ok(query_invoice_metadata_result) => query_invoice_metadata_result,
        Err(e) => {
            eprintln!(
                "Błąd podczas pobierania query_invoice_metadata_result: {}; {}",
                e.code, e.message
            );
            return;
        }
    };

    // if at least one invoice has been fetched
    if query_invoice_metadata_result.invoices.len() > 0 {
        if let Some(invoice_metadata) = query_invoice_metadata_result.invoices.first() {
            let nip = company_info.nip;
            let issue_date = invoice_metadata.invoicing_date;
            let invoice_hash = invoice_metadata.invoice_hash.to_owned();

            let qrcode = match client
                .get_qrcode(&nip, &issue_date, &invoice_hash, Some(300))
                .await
            {
                Ok(qrcode) => qrcode,
                Err(e) => {
                    eprintln!("Error: {}; {}", e.code, e.message);
                    return;
                }
            };

            std::fs::write("qr.png", qrcode).unwrap();
        }
    }
}
