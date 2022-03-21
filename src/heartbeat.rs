use crate::domain::Hub;
use serde::{Deserialize, Serialize};
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
    access_users: Vec<AccessUserResponseData>,
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

// https://serde.rs/custom-date-format.html
// JS Date.toJSON()
mod json_naive_date_time {
    use chrono::{NaiveDateTime, Timelike};
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(dt: &NaiveDateTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", dt.format("%Y-%m-%dT%H:%M:%S%.3fZ")); // JS Date.toJSON()
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let dt = s
            .parse::<chrono::DateTime<chrono::Utc>>()
            .map_err(serde::de::Error::custom)?;
        Ok(dt.naive_utc().with_nanosecond(0).unwrap())
    }

    #[test]
    fn test_json_naive_date_time() {
        #[derive(Debug, serde::Serialize, Deserialize)]
        struct S {
            #[serde(with = "self")]
            dt: chrono::NaiveDateTime,
        }
        let dt = chrono::NaiveDate::from_ymd(2001, 9, 8).and_hms(1, 46, 40);
        let data = S { dt };
        let json = serde_json::to_string(&data).unwrap();
        assert_eq!(json, r#"{"dt":"2001-09-08T01:46:40.000Z"}"#);

        let data: S = serde_json::from_str(&json).unwrap();
        assert_eq!(data.dt, dt);

        let result = serde_json::from_str::<S>(r#"{"dt":"2001-09-08T01:46:40.000"}"#);
        assert!(result.is_err());
    }
}

// https://stackoverflow.com/questions/44301748/how-can-i-deserialize-an-optional-field-with-custom-functions-using-serde
// JS Date.toJSON()
mod json_option_naive_date_time {
    use chrono::{NaiveDateTime, Timelike};
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(dt: &Option<NaiveDateTime>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if let Some(ref d) = *dt {
            return crate::heartbeat::json_naive_date_time::serialize(d, serializer);
        }
        serializer.serialize_none()
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<NaiveDateTime>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: Option<String> = Option::deserialize(deserializer)?;
        if let Some(s) = s {
            let dt = s
                .parse::<chrono::DateTime<chrono::Utc>>()
                .map_err(serde::de::Error::custom)?;
            return Ok(Some(dt.naive_utc().with_nanosecond(0).unwrap()));
        }
        Ok(None)
    }

    #[test]
    fn test_json_option_naive_date_time() {
        #[derive(serde::Serialize, Deserialize)]
        struct S {
            #[serde(with = "self")]
            opt_dt: Option<chrono::NaiveDateTime>,
        }
        let dt = chrono::NaiveDate::from_ymd(2001, 9, 8).and_hms(1, 46, 40);
        let opt_dt = Some(dt);
        let data = S { opt_dt };
        let json = serde_json::to_string(&data).unwrap();
        assert_eq!(json, r#"{"opt_dt":"2001-09-08T01:46:40.000Z"}"#);

        let data: S = serde_json::from_str(&json).unwrap();
        assert_eq!(data.opt_dt, opt_dt);

        let opt_dt = None;
        let data = S { opt_dt };
        let json = serde_json::to_string(&data).unwrap();
        assert_eq!(json, r#"{"opt_dt":null}"#);

        let data: S = serde_json::from_str(&json).unwrap();
        assert_eq!(data.opt_dt, opt_dt);

        let result = serde_json::from_str::<S>(r#"{"opt_dt":"2001-09-08T01:46:40.000"}"#);
        assert!(result.is_err());
    }
}

pub async fn heartbeat(host: String, pool: &SqlitePool) -> anyhow::Result<()> {
    let hub: Hub = sqlx::query_as("select id, name, cloudLastAccessEventAt from AccessHub")
        .fetch_one(pool)
        .await?;
    println!("{:#?}", hub);

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
            // cloud_last_access_event_at: Some(
            //     chrono::NaiveDate::from_ymd(2014, 5, 17).and_hms(7, 30, 23),
            // ),
            cloud_last_access_event_at: hub.cloud_last_access_event_at,
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

    if !res.status().is_success() {
        return Err(anyhow::anyhow!("Response error: {}", res.text().await?));
    }

    let data = res.json::<ResponseData>().await?;
    println!("response data: {:#?}", data);

    if hub.cloud_last_access_event_at == None
        || hub.cloud_last_access_event_at.unwrap() != data.access_hub.cloud_last_access_event_at
    {
        let rows_affected =
            sqlx::query(r#"update AccessHub set cloudLastAccessEventAt = ? where id = ?"#)
                .bind(data.access_hub.cloud_last_access_event_at)
                .bind(hub.id)
                .execute(pool)
                .await?
                .rows_affected();
        if rows_affected != 1 {
            return Err(anyhow::anyhow!(
                "Update cloud_last_access_event_at affected no rows"
            ));
        }
    }

    Ok(())
}
