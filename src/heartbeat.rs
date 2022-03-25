use crate::domain::{Hub, Point, User, Point2User};
use futures::TryStreamExt;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
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
}

#[derive(Debug, PartialEq)]
struct UserWithPointIds {
    user: User,
    point_ids: Vec<i64>,
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
    let hub: Hub = sqlx::query_as("select id, cloud_last_access_event_at from AccessHub")
        .fetch_one(pool)
        .await?;
    println!("{:#?}", hub);

    let events: Vec<AccessEventRequestData> = match hub.cloud_last_access_event_at {
        Some(cloud_last_access_event_at) => {
            // Leave margin to prevent race condition.
            sqlx::query_as(
                "select at, access, code, access_user_id, access_point_id from AccessEvent 
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

    if hub.id != data.access_hub.id {
        return Err(anyhow::anyhow!(
            "Hub id {} does not match cloud hub id {}",
            hub.id,
            data.access_hub.id
        ));
    }

    if hub.cloud_last_access_event_at == None
        || hub.cloud_last_access_event_at.unwrap() != data.access_hub.cloud_last_access_event_at
    {
        let rows_affected =
            sqlx::query(r#"update AccessHub set cloud_last_access_event_at = ? where id = ?"#)
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

    let mut local_points = HashMap::<i64, Point>::new();
    let mut rows =
        sqlx::query_as::<_, Point>(r#"select id, position from AccessPoint"#).fetch(pool);
    while let Some(u) = rows.try_next().await? {
        local_points.insert(u.id, u);
    }

    let invalid_point_ids: HashSet<i64> = data
        .access_hub
        .access_users
        .iter()
        .flat_map(|u| &u.access_points)
        .filter(|p| !local_points.contains_key(&p.id))
        .map(|p| p.id)
        .collect();
    if !invalid_point_ids.is_empty() {
        return Err(anyhow::anyhow!(
            "Invalid point ids in server response: {:#?}",
            invalid_point_ids
        ));
    }

    let mut user2points = HashMap::<i64, Vec<i64>>::new();
    let mut rows = sqlx::query_as::<_, Point2User>(
        r#"select access_user_id, access_point_id from AccessPointToAccessUser"#,
    )
    .fetch(pool);
    while let Some(u2p) = rows.try_next().await? {
        if let Some(points) = user2points.get_mut(&u2p.access_user_id) {
            points.push(u2p.access_point_id);
        } else {
            user2points.insert(u2p.access_user_id, vec![u2p.access_point_id]);
        }
    }

    let mut local_users = HashMap::<i64, UserWithPointIds>::new();
    let mut rows = sqlx::query_as::<_, User>(
        r#"select id, name, code, activate_code_at, expire_code_at from AccessUser"#,
    )
    .fetch(pool);
    while let Some(u) = rows.try_next().await? {
        let id = u.id;
        local_users.insert(
            id,
            UserWithPointIds {
                user: u,
                point_ids: user2points.remove(&id).unwrap_or_default(),
            },
        );
    }
    dbg!(&local_users);

    let mut cloud_users = HashMap::<i64, UserWithPointIds>::new();
    let cloud_users_len = data.access_hub.access_users.len();
    for cloud_user_data in data.access_hub.access_users {
        if cloud_user_data.code.is_empty() {
            return Err(anyhow::anyhow!(
                "Cloud user {} does not have code",
                cloud_user_data.id
            ));
        }
        if cloud_user_data.access_points.is_empty() {
            return Err(anyhow::anyhow!(
                "Cloud user {} does not have any points",
                cloud_user_data.id
            ));
        }
        cloud_users.insert(
            cloud_user_data.id,
            UserWithPointIds {
                user: User {
                    id: cloud_user_data.id,
                    name: cloud_user_data.name,
                    code: cloud_user_data.code,
                    activate_code_at: cloud_user_data.activate_code_at,
                    expire_code_at: cloud_user_data.expire_code_at,
                },
                point_ids: cloud_user_data
                    .access_points
                    .iter()
                    .map(|p| p.id)
                    .collect::<Vec<i64>>(),
            },
        );
    }

    if cloud_users.len() != cloud_users_len {
        return Err(anyhow::anyhow!("Duplicate cloud access user id's"));
    }

    let mut common_ids = HashSet::<i64>::new();
    let mut create_users = Vec::<&UserWithPointIds>::new();
    let mut update_users = Vec::<&UserWithPointIds>::new();
    let mut changed_codes = HashSet::<&str>::new();
    for cloud_user in cloud_users.values() {
        if let Some(local_user) = local_users.get(&cloud_user.user.id) {
            common_ids.insert(cloud_user.user.id);
            if local_user != cloud_user {
                update_users.push(cloud_user);
                if local_user.user.code != cloud_user.user.code {
                    changed_codes.insert(&cloud_user.user.code);
                }
            }
        } else {
            create_users.push(cloud_user);
        }
    }

    let delete_ids: HashSet<i64> = local_users
        .keys()
        .filter(|k| !common_ids.contains(k))
        .copied()
        .collect();

    let recycled_code_local_users: Vec<&UserWithPointIds> = update_users
        .iter()
        .flat_map(|x| local_users.get(&x.user.id))
        .filter(|x| changed_codes.contains(&*x.user.code))
        .collect();

    dbg!(&cloud_users);
    dbg!(&create_users);
    dbg!(&update_users);
    dbg!(&common_ids);
    dbg!(&delete_ids);
    dbg!(&changed_codes);
    dbg!(&recycled_code_local_users);

    // Access user codes must be unique: delete, update recyled codes, update, create.
    // TODO: Transaction
    let mut tx = pool.begin().await?;
    if !delete_ids.is_empty() {
        let query = format!(
            "delete from AccessUser where id in ({})",
            delete_ids
                .iter()
                .map(|_| "?")
                .collect::<Vec<&str>>()
                .join(", ")
        );
        let mut q = sqlx::query(&query);
        for id in delete_ids.iter() {
            q = q.bind(id);
        }
        let rows_affected = q.execute(&mut tx).await?.rows_affected();
        if rows_affected as usize != delete_ids.len() {
            return Err(anyhow::anyhow!(
                "Delete users affected {} rows instead of {}.",
                rows_affected,
                delete_ids.len()
            ));
        }
    }

    if !recycled_code_local_users.is_empty() {
        // TODO: Robus way to make recycled code unique.
        for u in recycled_code_local_users {
            let rows_affected = sqlx::query(r#"update AccessUser set code = ? where id = ?"#)
                .bind(format!("{}-", &u.user.code))
                .bind(u.user.id)
                .execute(&mut tx)
                .await?
                .rows_affected();
            if rows_affected != 1 {
                return Err(anyhow::anyhow!(
                    "Update user {} recyled code affected no rows",
                    u.user.id
                ));
            }
        }
    }

    if !update_users.is_empty() {
        for u in update_users {
            let rows_affected = sqlx::query(
                r#"update AccessUser set name=?, code=?, activate_code_at=?, expire_code_at=? where id=?"#)
                .bind(&u.user.name)
                .bind(&u.user.code)
                .bind(u.user.activate_code_at)
                .bind(u.user.expire_code_at)
                .bind(u.user.id)
                .execute(&mut tx)
                .await?
                .rows_affected();
            if rows_affected != 1 {
                return Err(anyhow::anyhow!(
                    "Update user {} affected no rows",
                    u.user.id
                ));
            }
            sqlx::query(r#"delete from AccessPointToAccessUser where access_user_id=?"#)
                .bind(u.user.id)
                .execute(&mut tx)
                .await?;
            if !u.point_ids.is_empty() {
                // insert or ignore?
                let query = format!(
                    r#"insert into AccessPointToAccessUser (access_user_id, access_point_id) values {}"#,
                    u.point_ids
                        .iter()
                        .map(|_| "(?,?)")
                        .collect::<Vec<&str>>()
                        .join(",")
                );
                let mut q = sqlx::query(&query);
                q = u
                    .point_ids
                    .iter()
                    .fold(q, |q, id| q.bind(u.user.id).bind(id));

                let rows_affected = q.execute(&mut tx).await?.rows_affected();
                if rows_affected as usize != u.point_ids.len() {
                    return Err(anyhow::anyhow!(
                        "Inserting user {} points affected {} rows instead of {}",
                        u.user.id,
                        rows_affected,
                        u.point_ids.len()
                    ));
                }
            }
        }
    }

    if !create_users.is_empty() {
        for u in create_users {
            let last_insert_rowid = sqlx::query(
            r#"insert into AccessUser (id, name, code, activate_code_at, expire_code_at) values (?, ?, ?, ?, ?)"#)
                .bind(u.user.id)
                .bind(&u.user.name)
                .bind(&u.user.code)
                .bind(u.user.activate_code_at)
                .bind(u.user.expire_code_at)
                .execute(&mut tx)
                .await?
                .last_insert_rowid();

            let query = format!(
                r#"insert into AccessPointToAccessUser (access_user_id, access_point_id) values {}"#,
                u.point_ids
                    .iter()
                    .map(|_| "(?,?)")
                    .collect::<Vec<&str>>()
                    .join(",")
            );
            let mut q = sqlx::query(&query);
            q = u
                .point_ids
                .iter()
                .fold(q, |q, id| q.bind(last_insert_rowid).bind(id));

            let rows_affected = q.execute(&mut tx).await?.rows_affected();
            if rows_affected as usize != u.point_ids.len() {
                return Err(anyhow::anyhow!(
                    "Inserting user {} points affected {} rows instead of {}",
                    u.user.id,
                    rows_affected,
                    u.point_ids.len()
                ));
            }
        }
    }
    tx.commit().await?;

    Ok(())
}
