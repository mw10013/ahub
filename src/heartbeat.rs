use serde_derive::{Deserialize, Serialize};
// use anyhow::Context;
// use sqlx::sqlite::SqlitePool;

#[derive(Deserialize)]
struct Ip {
    origin: String,
}

#[derive(Serialize, Deserialize)]
struct RequestData {
    accessHub: AccessHubRequestData,
}

#[derive(Serialize, Deserialize)]
struct AccessHubRequestData {
    id: i64,
    cloudLastAccessEventAt: Option<String>,
    accessEvents: Vec<AccessEventRequestData>,
}

#[derive(Serialize, Deserialize)]
struct AccessEventRequestData {
    at: String,
    access: String,
    code: String,
    accessUserId: Option<i64>,
    accessPointId: i64,
}

pub async fn heartbeat(host: String) -> anyhow::Result<()> {
    // select strftime('%Y-%m-%dT%H:%M:%fZ', 'now');
    // SELECT strftime('%s'); -- %s		seconds since 1970-01-01
    // SELECT strftime('%f'); -- %s		seconds since 1970-01-01
    // SELECT sqlite_version();
    /*
    let ip = reqwest::get("http://httpbin.org/ip")
        .await?
        .json::<Ip>()
        .await?;
    println!("ip: {}", ip.origin);
        */

    let request_data = RequestData {
        accessHub: AccessHubRequestData {
            id: 1,
            cloudLastAccessEventAt: None,
            accessEvents: vec![],
        },
    };
    let client = reqwest::Client::new();
    let res = client
        .post(format!("{}/api/accesshub/heartbeat", host))
        .json(&request_data)
        .send()
        .await?;
    // println!("{:#?}", res);

    if res.status().is_success() {
        let json: serde_json::Value = res.json().await?;
        println!("json: {:#?}", json);
    } else {
        let text = res.text().await?;
        println!("error: {}", text);
    }

    Ok(())
}
