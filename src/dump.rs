use crate::domain::{
    Event, Hub, Point, Point2User, PointWithRelations, User, User2Point, UserWithRelations,
};
use futures::TryStreamExt;
use sqlx::sqlite::SqlitePool;
use std::collections::HashMap;

pub async fn dump_hub(pool: &SqlitePool) -> anyhow::Result<()> {
    let hub: Hub = sqlx::query_as("select id, cloudLastAccessEventAt from AccessHub")
        .fetch_one(pool)
        .await?;
    println!("{:#?}", hub);
    Ok(())
}

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

pub async fn dump_users(take: i32, skip: i32, pool: &SqlitePool) -> anyhow::Result<()> {
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
        "select id, position from AccessPoint where id in ({})",
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

pub async fn dump_points(take: i32, skip: i32, pool: &SqlitePool) -> anyhow::Result<()> {
    let points = sqlx::query_as::<_, Point>(
        r#"select id, position from AccessPOint order by position asc limit ? offset ?"#,
    )
    .bind(take)
    .bind(skip)
    .fetch_all(pool)
    .await?;

    let point_ids: Vec<i64> = points.iter().map(|p| p.id).collect();
    let query = format!(
        "select A as point_id, B as user_id from _AccessPointToAccessUser where A in ({})",
        point_ids
            .iter()
            .map(|_| "?")
            .collect::<Vec<&str>>()
            .join(",")
    );
    let mut q = sqlx::query_as::<_, Point2User>(&query);
    for id in point_ids.iter() {
        q = q.bind(id);
    }

    let mut point2users = HashMap::<i64, Vec<i64>>::new();
    let mut rows = q.fetch(pool);
    while let Some(p2u) = rows.try_next().await? {
        if let Some(points) = point2users.get_mut(&p2u.user_id) {
            points.push(p2u.point_id);
        } else {
            point2users.insert(p2u.user_id, vec![p2u.point_id]);
        }
    }
    let point_ids: Vec<_> = point2users.values().flatten().copied().collect();

    let query = format!(
        "select id, name, code, activateCodeAt, expireCodeAt from AccessUser where id in ({})",
        point_ids
            .iter()
            .map(|_| "?")
            .collect::<Vec<&str>>()
            .join(", ")
    );
    let mut q = sqlx::query_as::<_, User>(&query);
    for id in point_ids.iter() {
        q = q.bind(id);
    }
    let mut users = HashMap::<i64, User>::new();
    let mut rows = q.fetch(pool);
    while let Some(u) = rows.try_next().await? {
        users.insert(u.id, u);
    }

    let points: Vec<PointWithRelations> = points
        .into_iter()
        .map(|p| {
            let id = p.id;
            PointWithRelations {
                point: p,
                users: match point2users.get(&id) {
                    Some(user_ids) => user_ids
                        .iter()
                        .flat_map(|id| users.get(id))
                        .cloned()
                        .collect(),
                    None => vec![],
                },
            }
        })
        .collect();

    // for u in &users {
    //     println!("{:#?}\n", u)
    // }
    dbg!(&points);

    Ok(())
}
