use anyhow::Context;
use sqlx::sqlite::SqlitePool;

use crate::domain::User;

pub async fn grant(user_id: i64, point_id: i64, pool: &SqlitePool) -> anyhow::Result<()> {
    println!("grant");
    /*
    CREATE TABLE IF NOT EXISTS "AccessEvent" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "at" DATETIME NOT NULL,
    "access" TEXT NOT NULL,
    "code" TEXT NOT NULL,
    "accessUserId" INTEGER,
    "accessPointId" INTEGER NOT NULL,
    CONSTRAINT "AccessEvent_accessPointId_fkey" FOREIGN KEY ("accessPointId") REFERENCES "AccessPoint" ("id") ON DELETE RESTRICT ON UPDATE CASCADE
    );

    SELECT id, name, code, activateCodeAt, expireCodeAt, accessHubId FROM AccessUser WHERE id = ? LIMIT ? OFFSET ?
    Params: [1,1,0]
    SELECT id, name, code, activateCodeAt, expireCodeAt, accessHubId FROM AccessUser WHERE id = 1;

    SELECT id, name, accessHubId, position FROM AccessPoint WHERE id = ? LIMIT ? OFFSET ?
    Params: [1,1,0]
    SELECT id, name, accessHubId, position FROM AccessPoint WHERE id = 1;

    BEGIN
    INSERT INTO AccessEvent (at, access, code, accessUserId, accessPointId) VALUES (?,?,?,?,?) RETURNING id
    Params: [2022-03-15 14:12:27.921 UTC,"grant","1111",1,1]
    INSERT INTO AccessEvent (at, access, code, accessUserId, accessPointId) VALUES ("2022-03-15 14:12:27.921 UTC","grant","1111",1,1) RETURNING id;
    INSERT INTO AccessEvent (at, access, code, accessUserId, accessPointId) VALUES (CURRENT_TIMESTAMP,"grant","1111",1,1);
    INSERT INTO AccessEvent (at, access, code, accessUserId, accessPointId) VALUES (CURRENT_TIMESTAMP,"grant", select code from AccessUser where id = 1,1,1);

    SELECT id, at, access, code, accessUserId, accessPointId FROM AccessEvent WHERE id = ? LIMIT ? OFFSET ?
    Params: [3,1,0]
    SELECT id, at, access, code, accessUserId, accessPointId FROM AccessEvent WHERE id = last_insert_rowid();
    COMMIT
     */

    let user = sqlx::query_as::<_, User>(
        r#"
select id, name, code, activateCodeAt, expireCodeAt from AccessUser where id = ?"#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
    .context("Access user does not exist")?;
    println!("user: {:?}", user);

    let id = sqlx::query!(
        r#"
        INSERT INTO AccessEvent (at, access, code, accessUserId, accessPointId) VALUES (CURRENT_TIMESTAMP,'grant', ?, ?, ?)       
        "#,
        user.code, user_id, point_id
    )
    .execute(pool)
    .await?
    .last_insert_rowid();
    println!("id: {}", id);

    Ok(())
}

pub async fn deny(point_id: i64, code: String, pool: &SqlitePool) -> anyhow::Result<()> {
    println!("deny");
    Ok(())
}
