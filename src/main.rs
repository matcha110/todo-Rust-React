mod handlers;
mod repositories;

use crate::handlers::create_todo;
use crate::repositories::{ TodoRepository, TodoRepositoryForMemory };

use axum::{ response::IntoResponse, extract::Extension, routing::{ get, post }, Router };
use std::{ env, sync::Arc };

#[tokio::main]
async fn main() {
    // loggingの初期化
    let log_level = env::var("RUST_LOG").unwrap_or("info".to_string());
    env::set_var("RUST_LOG", log_level);
    tracing_subscriber::fmt::init();

    let repository = TodoRepositoryForMemory::new();
    let app = create_app(repository);

    // axum::Serverはエラーが出るため下記のように変更
    // 参考記事：https://qiita.com/raiga0310/items/da2049912eae68f1a0c2
    let addr = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    axum::serve(addr, app).await.unwrap();
}

fn create_app<T: TodoRepository>(repository: T) -> Router {
    Router::new()
        .route("/", get(root))
        .route("/todos", post(create_todo::<T>))
        .layer(Extension(Arc::new(repository)))
}

async fn root() -> impl IntoResponse {
    "Hello, World!"
}

// 自動テストの追加
#[cfg(test)]
mod test {
    use super::*;
    use axum::{ body::Body, http::{ Request } };
    use tower::ServiceExt;

    #[tokio::test]
    async fn should_return_hello_world() {
        let repository = TodoRepositoryForMemory::new();
        let req = Request::builder().uri("/").body(Body::empty()).unwrap();
        let res = create_app(repository).oneshot(req).await.unwrap();
        let bytes = axum::body::to_bytes(res.into_body(), 1024 * 1024).await.unwrap();
        let body: String = String::from_utf8(bytes.to_vec()).unwrap();
        assert_eq!(body, "Hello, World!");
    }
}
