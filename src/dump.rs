use crate::domain::{Event, Point, User, User2Point, UserWithRelations};
use futures::TryStreamExt;
use sqlx::sqlite::SqlitePool;
use std::collections::HashMap;

// SELECT sqlite_version(); -> 3.37.0

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

    // dump: 3.37.0 released 2021-11-27, no unixepoch()
    // 2022-02-22 (3.38.0) has unixepoch()
    // 2022-03-12 (3.38.1) most recent version
    // libsqlite3-sys 0.24.1 should have bundled sqlite 3.38.0
    // libsqlite3-sys/sqlite3/bindgen_bundled_version.rs
    // bumped to 3.38.1 but not released yet https://github.com/rusqlite/rusqlite/commit/c3b419b1e53925c02e35a0dde019727153e1e6a8
    // sqlx has libsqlite3-sys 0.23.2
    // https://crates.io/crates/libsqlite3-sys/0.23.2
    //  currently SQLite 3.36.0 (as of rusqlite 0.26.0 / libsqlite3-sys 0.23.0).
    // https://github.com/rusqlite/rusqlite/releases
    //  libsqlite3-sys 0.24.1 (latest) has sqlite 3.38.0 bundled
    // libsqlite3-sys-v0.23.1: SQLITE_VERSION_NUMBER: i32 = 3036000;
    // 11/28/2021: https://github.com/rusqlite/rusqlite/commit/795a53d3682d5daf0b31f9a37eac4052c55558ca
    //  https://github.com/rusqlite/rusqlite/commit/795a53d3682d5daf0b31f9a37eac4052c55558ca

    let sqlite_version: (String,) = sqlx::query_as("select sqlite_version()")
        .fetch_one(pool)
        .await?;
    println!("sqlite_version: {}", sqlite_version.0);

    // let unix_epoch: (i64,) = sqlx::query_as("select unixepoch()").fetch_one(pool).await?;
    // println!("unix_epoch: {}", unix_epoch.0);

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
