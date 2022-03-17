use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
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
}

pub async fn heartbeat() -> anyhow::Result<()> {
    /*
    let mut map = HashMap::new();
    map.insert("lang", "rust");
    map.insert("body", "json");

    let client = reqwest::Client::new();
    let res = client
        .post("http://httpbin.org/post")
        .json(&map)
        .send()
        .await?;
    println!("{:#?}", res);

    let ip = reqwest::get("http://httpbin.org/ip")
        .await?
        .json::<Ip>()
        .await?;
    println!("ip: {}", ip.origin);
        */

    let heartbeat_request_data = HeartbeatRequestData {
        accessHub: HeartbeatAccessHubRequestData { id: 1 },
    };
    let client = reqwest::Client::new();
    let res = client
        // .post("http://localhost:3000/api/accesshub/heartbeat")
        .post("http://172.26.0.1:3000/api/accesshub/heartbeat")
        .json(&heartbeat_request_data)
        .send()
        .await?;
    println!("{:#?}", res);

    Ok(())
}
