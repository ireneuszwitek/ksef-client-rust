#[tokio::main]
async fn main() {
    let client = ksef::Client::new(ksef::Environment::Prod, 2000);
    let access_token = "<access_token>".to_string();

    let sessions_filter = ksef::session::SessionsFilter {
         statuses: Some(vec![ksef::session::SessionStatus::Succeeded]),
        ..Default::default()
    };

    let sessions = match client
        .get_sessions(ksef::session::SessionType::Batch, &access_token, 10, &Some(sessions_filter))
        .await
    {
        Ok(sessions) => sessions,
        Err(e) => {
            eprintln!("Error: {}; {}", e.code, e.message);
            return;
        }
    };

    println!("{:#?}", sessions)
}
