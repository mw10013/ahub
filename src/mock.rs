use crate::domain::{Event, User};
use anyhow::Context;
use sqlx::sqlite::SqlitePool;

pub async fn grant(user_id: i64, point_id: i64, pool: &SqlitePool) -> anyhow::Result<()> {
    let user = sqlx::query_as::<_, User>(
        r#"
select id, name, code, activate_code_at, expire_code_at from AccessUser where id = ?"#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
    .context("Access user does not exist")?;

    let id = sqlx::query!(
        r#"
        INSERT INTO AccessEvent (at, access, code, access_user_id, access_point_id) VALUES (CURRENT_TIMESTAMP,'grant', ?, ?, ?)       
        "#,
        user.code, user_id, point_id
    )
    .execute(pool)
    .await?
    .last_insert_rowid();
    print_event(id, pool).await
}

pub async fn deny(point_id: i64, code: String, pool: &SqlitePool) -> anyhow::Result<()> {
    let id = sqlx::query!(
        r#"
        INSERT INTO AccessEvent (at, access, code, access_point_id) VALUES (CURRENT_TIMESTAMP,'deny', ?, ?)           
        "#,
        code, point_id
    )
    .execute(pool)
    .await?
    .last_insert_rowid();
    print_event(id, pool).await
}

pub async fn swap(pool: &SqlitePool) -> anyhow::Result<()> {
    let users = sqlx::query_as::<_, User>(
        r#"select id, name, code, activate_code_at, expire_code_at from AccessUser order by id asc limit ?"#,
    )
    .bind(2)
    .fetch_all(pool)
    .await?;

    if let [u1, u2] = &users[..] {
        let rows_affected = sqlx::query("update AccessUser set code=? where id=?")
            .bind(format!("{}-", u1.code))
            .bind(u1.id)
            .execute(pool)
            .await?
            .rows_affected();
        if rows_affected != 1 {
            return Err(anyhow::anyhow!(
                "Updating user {} code is {}",
                u1.id,
                rows_affected
            ));
        }

        let rows_affected = sqlx::query("update AccessUser set code=? where id=?")
            .bind(&u1.code)
            .bind(u2.id)
            .execute(pool)
            .await?
            .rows_affected();
        if rows_affected != 1 {
            return Err(anyhow::anyhow!(
                "Updating user {} code is {}",
                u2.id,
                rows_affected
            ));
        }

        let rows_affected = sqlx::query("update AccessUser set code=? where id=?")
            .bind(&u2.code)
            .bind(u1.id)
            .execute(pool)
            .await?
            .rows_affected();
        if rows_affected != 1 {
            return Err(anyhow::anyhow!(
                "Updating user {} code is {}",
                u2.id,
                rows_affected
            ));
        }
    } else {
        return Err(anyhow::anyhow!("No access users to swap."));
    }

    let users = sqlx::query_as::<_, User>(
        r#"select id, name, code, activate_code_at, expire_code_at from AccessUser order by id asc limit ?"#,
    )
    .bind(2)
    .fetch_all(pool)
    .await?;
    dbg!(&users);

    Ok(())
}

async fn print_event(id: i64, pool: &SqlitePool) -> anyhow::Result<()> {
    let event = sqlx::query_as::<_, Event>(
        r#"
select id, at, access, code, access_user_id, access_point_id from AccessEvent where id = ?"#,
    )
    .bind(id)
    .fetch_one(pool)
    .await
    .context("Event does not exist")?;
    println!("{:#?}", event);
    Ok(())
}
