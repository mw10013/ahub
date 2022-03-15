use anyhow::Context;
use sqlx::sqlite::SqlitePool;

pub async fn heartbeat() -> anyhow::Result<()> {
    println!("heartbeat");
    Ok(())

}