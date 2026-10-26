use axum::{Json, Router, extract::State, http::StatusCode, routing::post};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Clone)]
struct AppState {
    db: SqlitePool,
}

#[derive(Deserialize)]
struct LoginPayload {
    username: String,
    password: String,
}

#[derive(Serialize)]
struct LoginResponse {
    message: String,
}

#[tokio::main]
async fn main() {
    let db = SqlitePool::connect("sqlite:users.db?mode=rwc")
        .await
        .unwrap();

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS users (
            username TEXT PRIMARY KEY, 
            password TEXT NOT NULL
        );",
    )
    .execute(&db)
    .await
    .unwrap();

    sqlx::query("INSERT OR IGNORE INTO users (username, password) VALUES (?, ?);")
        .bind("asdfg")
        .bind("asdfg")
        .execute(&db)
        .await
        .unwrap();

    let state = AppState { db };

    let app = Router::new()
        .route("/login", post(login_handler))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}

async fn login_handler(
    State(state): State<AppState>,
    Json(payload): Json<LoginPayload>,
) -> Result<Json<LoginResponse>, StatusCode> {
    let result: Option<(String,)> =
        sqlx::query_as("SELECT password FROM users WHERE username = ?;")
            .bind(&payload.username)
            .fetch_optional(&state.db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Some((stored_password,)) = result {
        if stored_password == payload.password {
            return Ok(Json(LoginResponse {
                message: format!("{}", payload.username),
            }));
        }
    }

    Err(StatusCode::UNAUTHORIZED)
}
