use serde::{Deserialize, Serialize};

use crate::domain::Hub;
// use anyhow::Context;
use sqlx::sqlite::SqlitePool;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RequestData {
    access_hub: AccessHubRequestData,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AccessHubRequestData {
    id: i64,
    #[serde(with = "json_option_naive_date_time")]
    cloud_last_access_event_at: Option<chrono::NaiveDateTime>,
    access_events: Vec<AccessEventRequestData>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
#[sqlx(rename_all = "camelCase")]
#[serde(rename_all = "camelCase")]
struct AccessEventRequestData {
    #[serde(with = "json_naive_date_time")]
    at: chrono::NaiveDateTime,
    access: String,
    code: String,
    access_user_id: Option<i64>,
    access_point_id: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ResponseData {
    access_hub: AccessHubResponseData,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AccessHubResponseData {
    id: i64,
    #[serde(with = "json_naive_date_time")]
    cloud_last_access_event_at: chrono::NaiveDateTime,
    // access_users: Vec<AccessUserResponseData>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AccessUserResponseData {
    id: i64,
    name: String,
    code: String,
    #[serde(with = "json_option_naive_date_time")]
    activate_code_at: Option<chrono::NaiveDateTime>,
    #[serde(with = "json_option_naive_date_time")]
    expire_code_at: Option<chrono::NaiveDateTime>,
    access_points: Vec<AccessPointResponseData>,
}

#[derive(Debug, Deserialize)]
struct AccessPointResponseData {
    id: i64,
    name: String,
}

#[test]
fn it_works() {
    let parse = "2017-04-07T11:11:23.348Z".parse::<chrono::NaiveDateTime>();
    println!("{:#?} {}", parse, parse.unwrap_err());
    let parse = "2017-04-07T11:11:23.348Z".parse::<chrono::DateTime<chrono::Utc>>();
    println!("{:#?}", parse);
}

// https://serde.rs/custom-date-format.html
// JS Date.toJSON()
mod json_naive_date_time {
    use chrono::NaiveDateTime;
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(dt: &NaiveDateTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", dt.format("%Y-%m-%dT%H:%M:%S%.3fZ"));
        println!("serialize: {} {}", dt, &s);
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        println!("deserialize: {}", &s);
        NaiveDateTime::parse_from_str(&s, "%+").map_err(serde::de::Error::custom)
    }
}

// https://stackoverflow.com/questions/44301748/how-can-i-deserialize-an-optional-field-with-custom-functions-using-serde
// JS Date.toJSON()
mod json_option_naive_date_time {
    use chrono::NaiveDateTime;
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(dt: &Option<NaiveDateTime>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if let Some(ref d) = *dt {
            return serializer.serialize_str(&d.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string());
        }
        serializer.serialize_none()
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<NaiveDateTime>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: Option<String> = Option::deserialize(deserializer)?;
        if let Some(s) = s {
            return Ok(Some(
                NaiveDateTime::parse_from_str(&s, "%+").map_err(serde::de::Error::custom)?,
            ));
        }
        Ok(None)
    }
}

pub async fn heartbeat(host: String, pool: &SqlitePool) -> anyhow::Result<()> {
    let hub: Hub = sqlx::query_as("select id, name, cloudLastAccessEventAt from AccessHub")
        .fetch_one(pool)
        .await?;
    println!("{:#?}", hub);

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
            cloud_last_access_event_at: Some(
                chrono::NaiveDate::from_ymd(2014, 5, 17).and_hms(7, 30, 22),
            ),
            // cloud_last_access_event_at: hub.cloud_last_access_event_at,
            access_events: events,
        },
    };
    println!("request_data: {:#?}", request_data);
    let client = reqwest::Client::new();
    let res = client
        .post(format!("{}/api/accesshub/heartbeat", host))
        .json(&request_data)
        .send()
        .await?;
    // println!("{:#?}", res);

    if res.status().is_success() {
        // let text = res.text().await?;
        // println!("text: {}", &text);
        // let data: ResponseData = serde_json::from_str(&text).unwrap();
        // let data: serde_json::Value = res.json().await?;
        let data = res.json::<ResponseData>().await?;
        println!("data: {:#?}", data);
    } else {
        let text = res.text().await?;
        println!("error: {}", text);
    }

    Ok(())
}
