#[derive(Debug, sqlx::FromRow)]
#[sqlx(rename_all = "camelCase")]
pub struct Hub {
    pub id: i64,
    pub name: String,
    pub cloud_last_access_event_at: Option<chrono::NaiveDateTime>,
}

#[derive(Debug)]
pub struct HubWithRelations {
    pub hub: Hub,
    pub points: Vec<Point>,
    pub users: Vec<UserWithRelations>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Point {
    pub id: i64,
    pub position: i64,
}

#[derive(Debug)]
pub struct PointWithRelations {
    pub point: Point,
    pub users: Vec<User>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct Point2User {
    pub point_id: i64,
    pub user_id: i64,
}

#[derive(PartialEq, Clone, Debug, sqlx::FromRow)]
#[sqlx(rename_all = "camelCase")]
pub struct User {
    pub id: i64,
    pub name: String,
    pub code: String,
    pub activate_code_at: Option<chrono::NaiveDateTime>,
    pub expire_code_at: Option<chrono::NaiveDateTime>,
}

#[derive(Debug)]
pub struct UserWithRelations {
    pub user: User,
    pub points: Vec<Point>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct User2Point {
    pub user_id: i64,
    pub point_id: i64,
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
