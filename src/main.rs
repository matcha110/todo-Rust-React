use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::env;

#[tokio::main]
async fn main() {
    // loggingの初期化
    let log_level = env::var("RUST_LOG").unwrap_or("info".to_string());
    env::set_var("RUST_LOG", log_level);
    tracing_subscriber::fmt::init();
    
    let app = Router::new()
        .route("/", get(root))
        .route("/users", post(create_user));
    // axum::Serverはエラーが出るため下記のように変更
    // 参考サイト：https://qiita.com/raiga0310/items/da2049912eae68f1a0c2
    let addr = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    axum::serve(addr, app)
        .await
        .unwrap();
}

async fn root() -> impl IntoResponse {
    "Hello, world!!!!!!"
}

async fn create_user(
    Json(payload): Json<CreateUser>,
) -> impl IntoResponse {
    let user = User {
        id: 123,
        username: payload.username,
    };
    // 戻り値
    (StatusCode::CREATED, Json(user))
}

// request
#[derive(Deserialize)]
struct CreateUser {
    username: String,
}

// resoinse
#[derive(Serialize)]
struct User {
    id: u64,
    username: String,
}