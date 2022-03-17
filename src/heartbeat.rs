use serde_derive::{Deserialize, Serialize};
// use anyhow::Context;
// use sqlx::sqlite::SqlitePool;

#[derive(Deserialize)]
struct Ip {
    origin: String,
}

#[derive(Serialize, Deserialize)]
struct HeartbeatRequestData {
    accessHub: HeartbeatAccessHubRequestData,
}

#[derive(Serialize, Deserialize)]
struct HeartbeatAccessHubRequestData {
    id: i64,
    cloudLastAccessEventAt: Option<String>,
    accessEvents: Vec<HeartbeatAccessEventRequestData>,
}

#[derive(Serialize, Deserialize)]
struct HeartbeatAccessEventRequestData {
    at: String,
    access: String,
    code: String,
    accessUserId: Option<i64>,
    accessPointId: i64,
}

pub async fn heartbeat(host: String) -> anyhow::Result<()> {
    /*
    let ip = reqwest::get("http://httpbin.org/ip")
        .await?
        .json::<Ip>()
        .await?;
    println!("ip: {}", ip.origin);
        */

    let heartbeat_request_data = HeartbeatRequestData {
        accessHub: HeartbeatAccessHubRequestData {
            id: 1,
            cloudLastAccessEventAt: None,
            accessEvents: vec![],
        },
    };
    let client = reqwest::Client::new();
    let res = client
        .post(format!("{}/api/accesshub/heartbeat", host))
        .json(&heartbeat_request_data)
        .send()
        .await?;
    println!("{:#?}", res);

    if res.status().is_success() {
        // let json = res.json().await?;
        // println!("json: {:#?}", json);
        let text = res.text().await?;
        println!("json: {}", text);
    } else {
        let text = res.text().await?;
        println!("error: {}", text);
    }

    Ok(())
}
