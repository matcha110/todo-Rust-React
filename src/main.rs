use axum::{ http::StatusCode, response::IntoResponse, routing::{ get, post }, Json, Router };
use serde::{ Deserialize, Serialize };
use std::env;

#[tokio::main]
async fn main() {
    // loggingの初期化
    let log_level = env::var("RUST_LOG").unwrap_or("info".to_string());
    env::set_var("RUST_LOG", log_level);
    tracing_subscriber::fmt::init();

    let app = create_app();
    // axum::Serverはエラーが出るため下記のように変更
    // 参考サイト：https://qiita.com/raiga0310/items/da2049912eae68f1a0c2
    let addr = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    axum::serve(addr, app).await.unwrap();
}

fn create_app() -> Router {
    Router::new().route("/", get(root)).route("/users", post(create_user))
}

async fn root() -> impl IntoResponse {
    "Hello, World!"
}

async fn create_user(Json(payload): Json<CreateUser>) -> impl IntoResponse {
    let user = User {
        id: 123,
        username: payload.username,
    };
    // 戻り値
    (StatusCode::CREATED, Json(user))
}

// request
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
struct CreateUser {
    username: String,
}

// resoinse
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
struct User {
    id: u64,
    username: String,
}

// 自動テストの追加
#[cfg(test)]
mod test {
    use super::*;
    use axum::{ body::Body, http::{ header, Method, Request } };
    use tower::ServiceExt;

    #[tokio::test]
    async fn should_return_hello_world() {
        let req = Request::builder().uri("/").body(Body::empty()).unwrap();
        let res = create_app().oneshot(req).await.unwrap();
        let bytes = axum::body::to_bytes(res.into_body(), 1024 * 1024).await.unwrap();
        let body: String = String::from_utf8(bytes.to_vec()).unwrap();
        assert_eq!(body, "Hello, World!");
    }

    #[tokio::test]
    async fn should_return_user_data() {
        let req = Request::builder()
            .uri("/users")
            .method(Method::POST)
            .header(header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
            .body(Body::from(r#"{ "username": "user0" }"#))
            .unwrap();

        let res = create_app().oneshot(req).await.unwrap();
        // typo:hyper::body::to_bytes -> axum::body::to_bytes
        // 第2引数を追記
        let bytes = axum::body::to_bytes(res.into_body(), 1024 * 1024).await.unwrap();
        let body: String = String::from_utf8(bytes.to_vec()).unwrap();
        let user: User = serde_json::from_str(&body).expect("cannot convert User instance.");
        assert_eq!(user, User {
            id: 123,
            username: "user0".to_string(),
        });
    }
}
