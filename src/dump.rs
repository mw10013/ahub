use std::collections::HashMap;

use futures::TryStreamExt;
use sqlx::sqlite::SqlitePool;

pub async fn dump_events(take: i32, skip: i32, pool: &SqlitePool) -> anyhow::Result<()> {
    let recs = sqlx::query!(
        r#"
                SELECT id, at, access, code, accessUserId, accessPointId
                FROM AccessEvent
                ORDER BY at DESC LIMIT ? OFFSET ?"#,
        take,
        skip
    )
    .fetch_all(pool)
    .await?;

    for rec in recs {
        println!("{:?}", rec);
    }

    Ok(())
}

#[derive(Debug)]
struct User {
    id: i64,
    name: String,
    code: String,
    activate_code_at: Option<chrono::NaiveDateTime>,
    expire_code_at: Option<chrono::NaiveDateTime>,
}

#[derive(sqlx::FromRow)]
struct User2Point {
    user_id: i64,
    point_id: i64,
}

#[derive(sqlx::FromRow)]
struct Point {
    id: i64,
    name: String,
}

pub async fn dump_users(
    take: i32,
    skip: i32,
    _swap: bool,
    pool: &SqlitePool,
) -> anyhow::Result<()> {
    // , activateCodeAt, expireCodeAt
    let users = sqlx::query_as!(
        User,
        r#"
select id, name, code, activateCodeAt as activate_code_at, expireCodeAt as expire_code_at
from AccessUser order by id asc limit ? offset ?"#,
        take,
        skip
    )
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

    let mut points = HashMap::<i64, (i64, String)>::new();
    let mut rows = q.fetch(pool);
    while let Some(p) = rows.try_next().await? {
        // let point_id: i64 = row.try_get("id")?;
        // let name: String = row.try_get("name")?;
        points.insert(p.id, (p.id, p.name));
    }

    for u in &users {
        println!("{:?}", *u);
        if let Some(point_ids) = user2points.get(&u.id) {
            println!(
                "  points: {:?}",
                point_ids
                    .iter()
                    .flat_map(|id| points.get(id))
                    .collect::<Vec<&(i64, String)>>()
            );
        }
    }

    Ok(())
}
