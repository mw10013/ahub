
#[derive(Debug)]
pub struct UserWithRelations<> {
    pub user: User,
    pub points: Vec<Point>,
}

#[derive(Debug, sqlx::FromRow)]
#[sqlx(rename_all = "camelCase")]
pub struct User {
    pub id: i64,
    pub name: String,
    pub code: String,
    pub activate_code_at: Option<chrono::NaiveDateTime>,
    pub expire_code_at: Option<chrono::NaiveDateTime>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct User2Point {
    pub user_id: i64,
    pub point_id: i64,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Point {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, sqlx::FromRow)]
#[sqlx(rename_all = "camelCase")]
pub struct Event {
    pub id: i64,
    pub at: chrono::NaiveDateTime,
    pub access: String,
    pub code: String,
    pub access_user_id: Option<i64>,
    pub access_point_id: i64,
}
