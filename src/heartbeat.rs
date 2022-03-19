use serde::{Deserialize, Serialize};

use crate::domain::Hub;
// use anyhow::Context;
use sqlx::sqlite::SqlitePool;

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

// toJSON(): '2022-03-18T18:00:53.188Z'
// sqlite: 2022-03-15 22:17:57
// NaiveDateTime: 2022-03-15T22:17:57
// format("%Y-%m-%dT%H:%M:%S.000Z")

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
#[sqlx(rename_all = "camelCase")]
#[serde(rename_all = "camelCase")]
struct AccessEventRequestData {
    // at: String,
    #[serde(with = "json_date_format")]
    at: chrono::NaiveDateTime,
    access: String,
    code: String,
    access_user_id: Option<i64>,
    access_point_id: i64,
}

mod json_date_format {
    use chrono::NaiveDateTime;
    use serde::{self, Deserialize, Deserializer, Serializer};

    // const FORMAT: &str = "%Y-%m-%d %H:%M:%S";
    const FORMAT: &str = "%Y-%m-%dT%H:%M:%S.000Z";

    pub fn serialize<S>(dt: &NaiveDateTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", dt.format(FORMAT));
        println!("serialize: {} {}", dt, &s);
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        // Utc.datetime_from_str(&s, FORMAT).map_err(serde::de::Error::custom)
        println!("deserialize: {}", &s);
        NaiveDateTime::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)
    }
}

pub async fn heartbeat(host: String, pool: &SqlitePool) -> anyhow::Result<()> {
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

    let hub: Hub = sqlx::query_as("select id, name, cloudLastAccessEventAt from AccessHub")
        .fetch_one(pool)
        .await?;
    println!("{:#?}", hub);

    let events: Vec<AccessEventRequestData> = sqlx::query_as(
        "select at, access, code, accessUserId, accessPointId from AccessEvent order by at desc",
    )
    .fetch_all(pool)
    .await?;

    dbg!(&events);

    let e = &events[0];
    println!("e: {:#?}", e);
    // println!(
    //     "at: {} {} {}",
    //     e.at,
    //     e.at.format("%Y-%m-%d %H:%M:%S"),
    //     e.at.format("%Y-%m-%dT%H:%M:%S.000Z")
    // );

    let json = serde_json::to_string_pretty(&e).unwrap();
    println!("json: {}", json);
    let data: AccessEventRequestData = serde_json::from_str(&json).unwrap();
    println!("data: {:#?}", data);
    println!("data.at: {}", data.at);


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
        // println!("json: {:#?}", json);
    } else {
        let text = res.text().await?;
        println!("error: {}", text);
    }

    Ok(())
}
