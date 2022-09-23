use axum::{
    response::{IntoResponse, Response},
    Json,
};
use chrono::Utc;
use clap::Parser;
use http::StatusCode;
use serde_json::json;
use sqlx::{sqlite::SqlitePoolOptions, Executor, Pool, Row, Sqlite};
use tracing::info;

#[derive(Clone, Debug, PartialEq, Eq, Parser)]
pub struct Options {
    /// SQLite database connections string.
    /// By default, it is a file named `storage.db` in the current directory.
    /// You can use `sqlite::memory:` to use an in-memory database.
    /// See <https://www.sqlite.org/uri.html>
    #[clap(long, env, default_value = "sqlite://storage.db")]
    database_url: String,
}

#[derive(Clone)]
pub struct PersistentStorage(Pool<Sqlite>);

#[derive(Debug)]
pub enum StorageError {
    DatabaseError(sqlx::error::Error),
}

pub async fn storage_client(options: &Options) -> PersistentStorage {
    info!(url = %&options.database_url, "Connecting to database");

    let db_pool = SqlitePoolOptions::new()
        .connect(&options.database_url)
        .await
        .expect("Unable to connect to DATABASE_URL");

    sqlx::migrate!().run(&db_pool).await.unwrap();

    PersistentStorage(db_pool)
}

impl IntoResponse for StorageError {
    fn into_response(self) -> Response {
        let message = match self {
            Self::DatabaseError(error) => error.to_string(),
        };
        let body = Json(json!({ "error": message }));
        (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
    }
}

impl PersistentStorage {
    pub async fn has_contributed(&self, uid: &str) -> Result<bool, StorageError> {
        let sql = "SELECT EXISTS(SELECT 1 FROM contributors WHERE uid = ?1)";
        self.0
            .fetch_one(sqlx::query(sql).bind(uid))
            .await
            .map(|row| row.get(0))
            .map_err(StorageError::DatabaseError)
    }

    pub async fn insert_contributor(&self, uid: &str) {
        let sql = "INSERT INTO contributors (uid, started_at) VALUES (?1, ?2)";
        self.0
            .execute(sqlx::query(sql).bind(uid).bind(Utc::now()))
            .await
            .ok();
    }

    pub async fn finish_contribution(&self, uid: &str) {
        let sql = "UPDATE contributors SET finished_at = ?1 WHERE uid = ?2";
        self.0
            .execute(sqlx::query(sql).bind(Utc::now()).bind(uid))
            .await
            .ok();
    }

    pub async fn expire_contribution(&self, uid: &str) {
        let sql = "UPDATE contributors SET expired_at = ?1 WHERE uid = ?2";
        self.0
            .execute(sqlx::query(sql).bind(Utc::now()).bind(uid))
            .await
            .ok();
    }
}
