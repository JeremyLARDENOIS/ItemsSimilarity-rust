use std::io::Write;

use dotenvy::dotenv;
use serde::Serialize;
use serde_json;
use sqlx::{postgres::PgPoolOptions, PgPool};

use crate::models::{History, Like, User, Video, UserJson, VideosJson, LikesJson, HistoryJson};

async fn get_user_ids(pool: &PgPool) -> Result<Vec<User>, sqlx::Error> {
    let rows = sqlx::query_as("SELECT id FROM users")
        .fetch_all(pool)
        .await?;

    Ok(rows)
}

async fn get_videos(pool: &PgPool) -> Result<Vec<Video>, sqlx::Error> {
    let rows = sqlx::query_as("SELECT id, title, description, \"publisherId\" FROM video")
        .fetch_all(pool)
        .await?;

    Ok(rows)
}

async fn get_likes(pool: &PgPool) -> Result<Vec<Like>, sqlx::Error> {
    let rows:Vec<Like> = sqlx::query_as("SELECT * FROM \"like\"")
        .fetch_all(pool)
        .await?;

    Ok(rows)
}

async fn get_history(pool: &PgPool) -> Result<Vec<History>, sqlx::Error> {
    let rows = sqlx::query_as("SELECT * FROM watchtime")
        .fetch_all(pool)
        .await?;

    Ok(rows)
}

async fn write_json<T: Serialize>(path: String, records: Vec<T>) -> Result<(), std::io::Error> {
    let mut file = std::fs::File::create(path).expect("Failed to create file");
    file.write_all(
        serde_json::to_string(&records)
            .expect("Failed to serialize user ids")
            .as_bytes(),
    )
    .expect("Failed to write to file");

    Ok(())
}

async fn users() {
    let url = std::env::var("USER_DATABASE_URL").expect("USER_DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&url)
        .await
        .expect("Failed to connect to user database");

    println!("Getting user ids");
    let ids = get_user_ids(&pool).await.expect("Failed to get user ids");
    let users_id = ids
        .iter()
        .map(|user| UserJson {
            user_id: user.id.clone().to_string(),
        })
        .collect::<Vec<UserJson>>();

    println!("Writing user ids");
    write_json("../user_ids.json".to_string(), users_id)
        .await
        .expect("Failed to write user ids");
}

async fn videos() {
    let url = std::env::var("VIDEO_DATABASE_URL").expect("VIDEO_DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&url)
        .await
        .expect("Failed to connect to video database");

    println!("Getting videos");
    let videos = get_videos(&pool).await.expect("Failed to get videos");
    let videos_json = videos
        .iter()
        .map(|video| VideosJson {
            video_id: video.id.clone().to_string(),
            title: video.title.clone(),
            description: video.description.clone(),
            publisher_id: video.publisherId.clone(),
        })
        .collect::<Vec<VideosJson>>();

    println!("Writing videos");
    write_json("../videos.json".to_string(), videos_json)
        .await
        .expect("Failed to write videos");
}

async fn likes() {
    let url = std::env::var("VIDEO_DATABASE_URL").expect("VIDEO_DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&url)
        .await
        .expect("Failed to connect to video database");

    println!("Getting likes");
    let likes = get_likes(&pool).await.expect("Failed to get likes");
    let likes_json = likes
        .iter()
        .map(|history| LikesJson {
            video_id: history.videoId.clone().to_string(),
            user_id: history.userId.clone().to_string(),
        })
        .collect::<Vec<LikesJson>>();

    println!("Writing likes");
    write_json("../likes.json".to_string(), likes_json)
        .await
        .expect("Failed to write likes");
}

async fn history() {
    let url = std::env::var("VIDEO_DATABASE_URL").expect("VIDEO_DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&url)
        .await
        .expect("Failed to connect to video database");

    println!("Getting history");
    let history = get_history(&pool).await.expect("Failed to get history");
    let history_json = history
        .iter()
        .map(|history| HistoryJson {
            user_id: history.userId.clone().to_string(),
            watch_time: history.watchedSeconds as f32,
            watch_percentage: history.watchedPercent as f32,
            is_watched: history.isWatched,
            video_id: history.videoId.clone().to_string(),
        })
        .collect::<Vec<HistoryJson>>();

    println!("Writing history");
    write_json("../history.json".to_string(), history_json)
        .await
        .expect("Failed to write history");
}

pub async fn dump() -> Result<(), sqlx::Error> {
    dotenv().ok();

    // Collect data from databases
    users().await;
    videos().await;
    likes().await;
    history().await;

    Ok(())
}
