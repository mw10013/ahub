use crate::domain::Hub;
// use anyhow::Context;
use sqlx::{Connection, SqliteConnection};

pub async fn token(set: &str, database_url: &str) -> anyhow::Result<()> {
    let mut conn = SqliteConnection::connect(&database_url).await?;

    let hub: Hub =
        sqlx::query_as("select id, api_token, cloud_last_access_event_at from AccessHub")
            .fetch_one(&mut conn)
            .await?;
    if set.is_empty() {
        // println!("{:#?}", hub);
        println!("token: {}", hub.api_token)
    } else {
        let rows_affected = sqlx::query("update AccessHub set api_token=? where id=?")
            .bind(set)
            .bind(&hub.id)
            .execute(&mut conn)
            .await?
            .rows_affected();
        if rows_affected != 1 {
            return Err(anyhow::anyhow!("Error updating token"));
        }
    }

    Ok(())
}
