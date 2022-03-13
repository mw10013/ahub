use sqlx::{sqlite::SqlitePool, Row};
use futures::TryStreamExt;

pub async fn dump_users(take: i32, skip: i32, swap: bool, pool: &SqlitePool) -> anyhow::Result<()> {
    println!("dump_users: take: {} skip: {} swap: {}", take, skip, swap);
    // select id, name, code, activateCodeAt, expirecodeAt from AccessUser order by id asc limit 2;
    // select B, A from _AccessPointToAccessUser where B in (1, 2);
    // select id, name from AccessPoint where id in (1,2,3,4,5,6,7,8,1,2,5,6);

    let users = sqlx::query!(
        r#"
select id, name, code, activateCodeAt, expireCodeAt
from AccessUser order by id asc limit ? offset ?"#,
        take,
        skip
    )
    .fetch_all(pool)
    .await?;

    for u in &users {
        println!("{:?}", *u);
    }

    let user_ids: Vec<i64> = users.iter().map(|u| u.id).collect();
    for ui in &user_ids {
        println!("user id: {}", ui);
    }

    let query = format!(
        "select B, A from _AccessPointToAccessUser where B in ({})",
        (0..user_ids.len())
            .map(|_| "?")
            .collect::<Vec<&str>>()
            .join(", ")
    );
    println!("query: {}", query);

    let mut q = sqlx::query(&query);
    for id in user_ids.iter() {
        q = q.bind(id);
    }

    let mut rows = q.fetch(pool);
    while let Some(row) = rows.try_next().await? {
        let a: i64 = row.try_get("A")?;
        let b: i64 = row.try_get("B")?;
        println!("B (user): {} A (point): {}", b, a)
    }

    Ok(())
}
