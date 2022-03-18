use crate::domain::{Event, Point, User, User2Point, UserWithRelations};
use futures::TryStreamExt;
use sqlx::sqlite::SqlitePool;
use std::collections::HashMap;

pub async fn dump_sqlite_version(pool: &SqlitePool) -> anyhow::Result<()> {
    let sqlite_version: (String,) = sqlx::query_as("select sqlite_version()")
        .fetch_one(pool)
        .await?;
    println!("sqlite_version: {}", sqlite_version.0);
    Ok(())
}

pub async fn dump_events(take: i32, skip: i32, pool: &SqlitePool) -> anyhow::Result<()> {
    let events = sqlx::query_as::<_, Event>(
        r#"
        select id, at, access, code, accessUserId, accessPointId from AccessEvent order by at desc limit ? offset ?
        "#
    )
    .bind(take)
    .bind(skip)
    .fetch_all(pool)
    .await?;

    for e in events {
        println!("{:#?}", e);
    }
    Ok(())
}

pub async fn dump_users(
    take: i32,
    skip: i32,
    _swap: bool,
    pool: &SqlitePool,
) -> anyhow::Result<()> {
    let users = sqlx::query_as::<_, User>(
        r#"
select id, name, code, activateCodeAt, expireCodeAt
from AccessUser order by id asc limit ? offset ?"#,
    )
    .bind(take)
    .bind(skip)
    .fetch_all(pool)
    .await?;

    let user_ids: Vec<i64> = users.iter().map(|u| u.id).collect();
    let query = format!(
        "select B as user_id, A as point_id from _AccessPointToAccessUser where B in ({})",
        user_ids
            .iter()
            .map(|_| "?")
            .collect::<Vec<&str>>()
            .join(", ")
    );
    let mut q = sqlx::query_as::<_, User2Point>(&query);
    for id in user_ids.iter() {
        q = q.bind(id);
    }

    let mut user2points = HashMap::<i64, Vec<i64>>::new();
    let mut rows = q.fetch(pool);
    while let Some(u2p) = rows.try_next().await? {
        if let Some(points) = user2points.get_mut(&u2p.user_id) {
            points.push(u2p.point_id);
        } else {
            user2points.insert(u2p.user_id, vec![u2p.point_id]);
        }
    }
    let point_ids: Vec<_> = user2points.values().flatten().copied().collect();

    let query = format!(
        "select id, name from AccessPoint where id in ({})",
        point_ids
            .iter()
            .map(|_| "?")
            .collect::<Vec<&str>>()
            .join(", ")
    );
    let mut q = sqlx::query_as::<_, Point>(&query);
    for id in point_ids.iter() {
        q = q.bind(id);
    }
    let mut points = HashMap::<i64, Point>::new();
    let mut rows = q.fetch(pool);
    while let Some(p) = rows.try_next().await? {
        points.insert(p.id, p);
    }

    let users: Vec<UserWithRelations> = users
        .into_iter()
        .map(|u| {
            let id = u.id;
            UserWithRelations {
                user: u,
                points: match user2points.get(&id) {
                    Some(point_ids) => point_ids
                        .iter()
                        .flat_map(|id| points.get(id))
                        .cloned()
                        .collect(),
                    None => vec![],
                },
            }
        })
        .collect();

    for u in &users {
        println!("{:#?}\n", u)
    }

    Ok(())
}
