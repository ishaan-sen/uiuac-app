use axum::{Json, Router, extract::State, http::StatusCode, routing::post};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::process::Stdio;
use tokio::{io::AsyncWriteExt, process::Command};

#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
enum Emit {
    Uir,
    Tir,
    Uasm,
}

impl Emit {
    fn as_str(&self) -> &'static str {
        match self {
            Emit::Uir => "uir",
            Emit::Tir => "tir",
            Emit::Uasm => "uasm",
        }
    }
}

#[derive(Deserialize)]
struct CompilePayload {
    code: String,
    emit: Emit,
}

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
        .route("/compile", post(compile_handler))
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

    if let Some((stored_password,)) = result
        && stored_password == payload.password
    {
        return Ok(Json(LoginResponse {
            message: payload.username.to_string(),
        }));
    }

    Err(StatusCode::UNAUTHORIZED)
}

async fn compile_handler(
    Json(payload): Json<CompilePayload>,
) -> Result<String, (StatusCode, String)> {
    let mut child = Command::new("uiuac")
        .args(["--emit", payload.emit.as_str(), "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("failed to start uiuac: {e}"),
            )
        })?;

    let mut stdin = child.stdin.take().unwrap();
    let code = payload.code;
    let writer = tokio::spawn(async move {
        let _ = stdin.write_all(code.as_bytes()).await;
        let _ = stdin.write_all(b"\n").await;
    });

    let output = child.wait_with_output().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("failed to run uiuac: {e}"),
        )
    })?;

    let _ = writer.await;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            String::from_utf8_lossy(&output.stderr).into_owned(),
        ))
    }
}
