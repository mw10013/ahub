use serde::{Deserialize, Serialize};

use crate::domain::Hub;
// use anyhow::Context;
use sqlx::sqlite::SqlitePool;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RequestData {
    access_hub: AccessHubRequestData,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AccessHubRequestData {
    id: i64,
    cloud_last_access_event_at: Option<chrono::NaiveDateTime>,
    access_events: Vec<AccessEventRequestData>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
#[sqlx(rename_all = "camelCase")]
#[serde(rename_all = "camelCase")]
struct AccessEventRequestData {
    #[serde(with = "json_date_format")]
    at: chrono::NaiveDateTime,
    access: String,
    code: String,
    access_user_id: Option<i64>,
    access_point_id: i64,
}

// https://serde.rs/custom-date-format.html
mod json_date_format {
    use chrono::NaiveDateTime;
    use serde::{self, Deserialize, Deserializer, Serializer};

    const FORMAT: &str = "%Y-%m-%dT%H:%M:%S.000Z"; // JS Date.toJSON()

    pub fn serialize<S>(dt: &NaiveDateTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", dt.format(FORMAT));
        // println!("serialize: {} {}", dt, &s);
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        // println!("deserialize: {}", &s);
        NaiveDateTime::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)
    }
}

pub async fn heartbeat(host: String, pool: &SqlitePool) -> anyhow::Result<()> {
    // DATETIME(current_timestamp, '-5 minutes')
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

    // let events: Vec<AccessEventRequestData> = sqlx::query_as(
    //     "select at, access, code, accessUserId, accessPointId from AccessEvent order by at desc",
    // )
    // .fetch_all(pool)
    // .await?;

    // select * from AccessEvent where at > '2022-03-15 23:00:00' and at < DATETIME(CURRENT_TIMESTAMP, '-5 seconds') order by at desc;
    let events: Vec<AccessEventRequestData> = match hub.cloud_last_access_event_at {
        Some(cloud_last_access_event_at) => {
            // Leave margin to prevent race condition.
            sqlx::query_as(
                "select at, access, code, accessUserId, accessPointId from AccessEvent 
                where at > ? and at < DATETIME(CURRENT_TIMESTAMP, '-5 seconds') order by at desc",
            )
            .bind(cloud_last_access_event_at)
            .fetch_all(pool)
            .await?
        }
        None => vec![],
    };
    dbg!(&events);

    let request_data = RequestData {
        access_hub: AccessHubRequestData {
            id: hub.id,
            cloud_last_access_event_at: hub.cloud_last_access_event_at,
            access_events: events,
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
