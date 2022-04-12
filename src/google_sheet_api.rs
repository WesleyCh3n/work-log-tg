use chrono::Local;
use google_sheets4 as sheets4;
use sheets4::api::ValueRange;
use sheets4::{hyper, hyper_rustls, oauth2, Sheets};
use std::collections::HashMap;
use std::env;

use crate::weather_api::get_weather;

pub async fn get_hub() -> Sheets {
    let secret = oauth2::read_service_account_key("./credential.json")
        .await
        .expect("./credential.json");
    let auth = oauth2::ServiceAccountAuthenticator::builder(secret)
        .build()
        .await
        .expect("fail to create auth");

    let hub = Sheets::new(
        hyper::Client::builder()
            .build(hyper_rustls::HttpsConnector::with_native_roots()),
        auth,
    );
    hub
}

pub async fn check(
    hub: &Sheets,
    check_status: String,
) -> Result<String, Box<dyn std::error::Error>> {
    let now = Local::now();
    let date = format!("{}", now.format("%F"));
    let time = format!("{}", now.format("%T"));
    let ws = get_weather().await?;

    println!(" 🌱 Check {} ...", &check_status);
    let pretty_msg = pretty_status(&date, &time, &ws);

    let row: Vec<Vec<String>> = vec![[
        date,
        time,
        check_status.to_owned(),
        ws["wx"].to_owned(),
        ws["pop"].to_owned(),
        ws["min_t"].to_owned(),
        ws["max_t"].to_owned(),
        ws["ci"].to_owned(),
    ]
    .iter()
    .cloned()
    .collect()];

    let req = ValueRange {
        major_dimension: Some("ROWS".into()),
        range: Some("sheet1".into()),
        values: Some(row),
    };

    hub.spreadsheets()
        .values_append(
            req,
            env::var("GOOGLE_SHEET_KEY")
                .expect("google sheet key failed")
                .as_str(),
            "sheet1",
        )
        .value_input_option("USER_ENTERED")
        .doit()
        .await?;
    println!(" ✨ Check {} Complete", &check_status);
    Ok(pretty_msg)
}

fn pretty_status(
    date: &String,
    time: &String,
    ws: &HashMap<&str, String>,
) -> String {
    let mut v: Vec<String> = Vec::new();
    v.push(format!("📅 {: <}\n", &date));
    v.push(format!("⏱ Time: {: <}\n", &time));
    v.push(format!("☀️ Stat: {: <}\n", &ws["wx"]));
    v.push(format!("🌦 Rain: {} {: <}\n", &ws["pop"], "%"));
    v.push(format!("🌡 MaxT: {} {: <}\n", &ws["max_t"], "℃"));
    v.push(format!("🌡 MinT: {} {: <}\n", &ws["min_t"], "℃"));
    v.push(format!("✨ Comf: {: <}\n", &ws["ci"]));
    v.join("")
}
