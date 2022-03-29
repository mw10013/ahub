use sqlx::{Connection, SqliteConnection};

use crate::domain::{ActiveCode, Point};

pub async fn access(code: &str, position: i64, database_url: &str) -> anyhow::Result<()> {
    if position < 1 {
        return Err(anyhow::anyhow!(
            "Position is 1-based and must be greater than 0."
        ));
    }
    let mut conn = SqliteConnection::connect(database_url).await?;
    let active_code = sqlx::query_as!(
        ActiveCode,
        r#"select access_point_id, position, code, access_user_id, activate_code_at, expire_code_at
        from ActiveCode where code = ? and position = ?"#,
        code,
        position
    )
    .fetch_optional(&mut conn)
    .await?;

    match active_code {
        Some(active_code) => {
            let _id = sqlx::query!(
            r#"
            INSERT INTO AccessEvent (at, access, code, access_user_id, access_point_id) VALUES (CURRENT_TIMESTAMP,'grant', ?, ?, ?)       
            "#,
            active_code.code, active_code.access_user_id, active_code.access_point_id)
                .execute(&mut conn)
                .await?
                .last_insert_rowid();
            println!("GRANT");
        }
        None => {
            let point = sqlx::query_as!(
                Point,
                r#"select id, position from AccessPoint where position = ?"#,
                position
            )
            .fetch_optional(&mut conn)
            .await?;
            match point {
                Some(point) => {
                    let _id = sqlx::query!(
                        r#"
                        INSERT INTO AccessEvent (at, access, code, access_point_id) VALUES (CURRENT_TIMESTAMP,'deny', ?, ?)       
                        "#,
                        code, point.id)
                            .execute(&mut conn)
                            .await?
                            .last_insert_rowid();
                    println!("DENY")
                }
                None => {
                    return Err(anyhow::anyhow!("Position {} does not exist", position));
                }
            }
        }
    }
    Ok(())
}
