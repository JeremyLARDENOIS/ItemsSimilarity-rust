use serde::{Serialize, Deserialize};
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow, Clone)]
pub struct User {
    pub id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserJson {
    pub user_id: String,
}

#[derive(Debug, sqlx::FromRow, Clone)]
pub struct Video {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub publisherId: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideosJson {
    pub video_id: String,
    pub title: String,
    pub description: String,
    pub publisher_id: String,
}

#[derive(Debug, sqlx::FromRow, Clone)]
pub struct Like {
    pub userId: String,
    pub videoId: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LikesJson {
    pub video_id: String,
    pub user_id: String,
}

#[derive(Debug, sqlx::FromRow, Clone)]
pub struct History {
    pub userId: String,
    pub watchedSeconds: f64,
    pub watchedPercent: f64,
    pub isWatched: bool,
    pub videoId: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryJson {
    pub user_id: String,
    pub watch_time: f32,
    pub watch_percentage: f32,
    pub is_watched: bool,
    pub video_id: String,
}