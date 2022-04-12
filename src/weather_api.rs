use reqwest::Url;
use std::collections::HashMap;

pub async fn get_weather<'a>(
) -> Result<HashMap<&'a str, String>, Box<dyn std::error::Error>> {
    let wx_key =
        std::env::var("WEATHER_API_KEY").expect("WEATHER_API_KEY not found");

    let url = Url::parse_with_params(
        "https://opendata.cwb.gov.tw/api/v1/rest/datastore/F-C0032-001",
        &[
            ("Authorization", wx_key.as_str()),
            ("locationName", "臺北市"),
        ],
    )?;

    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .header(reqwest::header::ACCEPT, "application/json")
        .send()
        .await?;

    let data = response.text().await?;
    let v: serde_json::Value = serde_json::from_str(&data)?;
    let ws_data = &v["records"]["location"][0]["weatherElement"];
    // let mut parsed = [(); 5].map(|_| String::new());

    let mut ws: HashMap<&str, String> = HashMap::new();
    for (i, name) in ["wx", "pop", "min_t", "ci", "max_t"].iter().enumerate() {
        let s = ws_data[i]["time"][0]["parameter"]["parameterName"].to_string();
        ws.insert(name, s[1..s.len() - 1].to_owned());
    }
    Ok(ws)
}
