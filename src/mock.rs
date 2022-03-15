use sqlx::sqlite::SqlitePool;

pub async fn grant(user_id: i64, point_id: i64, pool: &SqlitePool) -> anyhow::Result<()> {
    println!("grant");

    Ok(())
}

pub async fn deny(point_id: i64, code: String, pool: &SqlitePool) -> anyhow::Result<()> {
    println!("deny");
    Ok(())
}
