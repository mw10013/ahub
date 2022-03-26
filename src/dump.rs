use crate::domain::{Event, Hub, Point, Point2User, PointWithRelations, User, UserWithRelations};
use futures::TryStreamExt;
use sqlx::{SqliteConnection};
use std::collections::HashMap;

pub async fn dump_hub(conn: &mut SqliteConnection) -> anyhow::Result<()> {
    let hub: Hub = sqlx::query_as("select id, cloud_last_access_event_at from AccessHub")
        .fetch_one(conn)
        .await?;
    println!("{:#?}", hub);
    Ok(())
}

pub async fn dump_sqlite_version(conn: &mut SqliteConnection) -> anyhow::Result<()> {
    let sqlite_version: (String,) = sqlx::query_as("select sqlite_version()")
        .fetch_one(conn)
        .await?;
    println!("sqlite_version: {}", sqlite_version.0);
    Ok(())
}

pub async fn dump_events(take: i32, skip: i32, conn: &mut SqliteConnection) -> anyhow::Result<()> {
    let events = sqlx::query_as::<_, Event>(
        r#"
        select id, at, access, code, access_user_id, access_point_id from AccessEvent order by at desc limit ? offset ?
        "#
    )
    .bind(take)
    .bind(skip)
    .fetch_all(&mut *conn)
    .await?;

    for e in events {
        println!("{:#?}", e);
    }
    Ok(())
}

pub async fn dump_users(take: i32, skip: i32, conn: &mut SqliteConnection) -> anyhow::Result<()> {
    let users = sqlx::query_as::<_, User>(
        r#"select id, name, code, activate_code_at, expire_code_at from AccessUser order by id asc limit ? offset ?"#,
    )
    .bind(take)
    .bind(skip)
    .fetch_all(&mut *conn)
    .await?;

    let user_ids: Vec<i64> = users.iter().map(|u| u.id).collect();
    let query = format!(
        "select access_user_id, access_point_id from AccessPointToAccessUser where access_user_id in ({})",
        user_ids
            .iter()
            .map(|_| "?")
            .collect::<Vec<&str>>()
            .join(", ")
    );
    let mut q = sqlx::query_as::<_, Point2User>(&query);
    for id in user_ids.iter() {
        q = q.bind(id);
    }

    let mut user2points = HashMap::<i64, Vec<i64>>::new();
    {
        let mut rows = q.fetch(&mut *conn);
        while let Some(u2p) = rows.try_next().await? {
            if let Some(points) = user2points.get_mut(&u2p.access_user_id) {
                points.push(u2p.access_point_id);
            } else {
                user2points.insert(u2p.access_user_id, vec![u2p.access_point_id]);
            }
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
    {
        let mut rows = q.fetch(&mut *conn);
        while let Some(p) = rows.try_next().await? {
            points.insert(p.id, p);
        }
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

pub async fn dump_points(take: i32, skip: i32, conn: &mut SqliteConnection) -> anyhow::Result<()> {
    let points = sqlx::query_as::<_, Point>(
        r#"select id, position from AccessPOint order by position asc limit ? offset ?"#,
    )
    .bind(take)
    .bind(skip)
    .fetch_all(&mut *conn)
    .await?;

    let point_ids: Vec<i64> = points.iter().map(|p| p.id).collect();
    let query = format!(
        "select access_point_id, access_user_id from AccessPointToAccessUser where access_point_id in ({})",
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
    {
        let mut rows = q.fetch(&mut *conn);
        while let Some(p2u) = rows.try_next().await? {
            if let Some(points) = point2users.get_mut(&p2u.access_user_id) {
                points.push(p2u.access_point_id);
            } else {
                point2users.insert(p2u.access_user_id, vec![p2u.access_point_id]);
            }
        }
    }

    let point_ids: Vec<_> = point2users.values().flatten().copied().collect();

    let query = format!(
        "select id, name, code, activate_code_at, expire_code_at from AccessUser where id in ({})",
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
    {
        let mut rows = q.fetch(&mut *conn);
        while let Some(u) = rows.try_next().await? {
            users.insert(u.id, u);
        }
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
