use sqlx::{Connection, SqliteConnection};

use crate::domain::{ActiveCode, Point};

pub async fn access(code: &str, position: i64, database_url: &str) -> anyhow::Result<()> {
    if position < 1 {
        return Err(anyhow::anyhow!(
            "Position is 1-based and must be greater than 0."
        ));
    }
    let mut conn = SqliteConnection::connect(database_url).await?;
    let code = sqlx::query_as::<_, ActiveCode>(
        r#"select access_point_id, position, code, access_user_id, activate_code_at, expire_code_at
        from ActiveCode where code = ? and position = ?"#,
    )
    .bind(code)
    .bind(position)
    .fetch_optional(&mut conn)
    .await?;

    if code.is_some() {
        println!("GRANT");
    }
    else {
        let point = sqlx::query_as::<_, Point>(
            r#"select id, position from AccessPoint where position = ?"#)
            .bind(position)
            .fetch_optional(&mut conn)
            .await?;
        if point.is_none() {
            return Err(anyhow::anyhow!("Position {} does not exist", position));
        }
        println!("DENY")
    }

    Ok(())
}
