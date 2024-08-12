use anyhow::Context;
use axum::{
    http::StatusCode,
    response::IntoResponse,
    extract::Extension,
    routing::{ get, post },
    Json,
    Router,
};
use serde::{ Deserialize, Serialize };
use std::{ env, sync::{ Arc, RwLock, RwLockReadGuard, RwLockWriteGuard } };
use thiserror::Error;
use std::collections::HashMap;

// use crate::handlers::create_todo;
// use crate::repositories::{ TodoRepository, TodoRepositoryForMemory };

// リポジトリで発生しうるエラーを定義
#[derive(Debug, Error)]
enum RepositoryError {
    #[error("NotFound, id is {0}")] NotFound(i32),
}

pub trait TodoRepository: Clone + std::marker::Send + std::marker::Sync + 'static {
    fn create(&self, payload: CreateTodo) -> Todo;
    fn find(&self, id: i32) -> Option<Todo>;
    fn all(&self) -> Vec<Todo>;
    fn update(&self, id: i32, payload: UpdateTodo) -> anyhow::Result<Todo>;
    fn delete(&self, id: i32) -> anyhow::Result<()>;
}

// Todoに必要な構造体を定義
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Todo {
    id: i32,
    text: String,
    completed: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct CreateTodo {
    text: String,
}

// Option<T>型は 取得できないかもしれない値 を表現する列挙型であり、値が無いことを示すNoneとあることを示すSome(T)のどちらかをとる
// cf, Result<T,E>は失敗するかもしれない処理の結果を表現する列挙型である。適切な使い分けが必要
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct UpdateTodo {
    text: Option<String>,
    completed: Option<bool>,
}

impl Todo {
    pub fn new(id: i32, text: String) -> Self {
        Self {
            id,
            text,
            completed: false,
        }
    }
}

type TodoDatas = HashMap<i32, Todo>;

#[derive(Debug, Clone)]
pub struct TodoRepositoryForMemory {
    store: Arc<RwLock<TodoDatas>>,
}

impl TodoRepositoryForMemory {
    pub fn new() -> Self {
        TodoRepositoryForMemory {
            store: Arc::default(),
        }
    }

    // write権限を持つHashMapをスレッドセーフに取得
    fn write_store_ref(&self) -> RwLockWriteGuard<TodoDatas> {
        self.store.write().unwrap()
    }

    // read権限を持つHashMapをスレッドセーフに取得
    fn read_store_ref(&self) -> RwLockReadGuard<TodoDatas> {
        self.store.read().unwrap()
    }
}

impl TodoRepository for TodoRepositoryForMemory {
    fn create(&self, payload: CreateTodo) -> Todo {
        let mut store = self.write_store_ref();
        let id = (store.len() + 1) as i32; //as：型キャスト
        let todo = Todo::new(id, payload.text.clone());
        store.insert(id, todo.clone());
        todo
    }

    fn find(&self, id: i32) -> Option<Todo> {
        let store = self.read_store_ref();
        store.get(&id).map(|todo| todo.clone()) //所有権を持っていないため戻り値にはCloneした値を設定
        // パフォーマンスを改善したい場合にはBOXを使用すると良い
    }

    fn all(&self) -> Vec<Todo> {
        let store = self.read_store_ref();
        Vec::from_iter(store.values().map(|todo| todo.clone()))
    }

    fn update(&self, id: i32, payload: UpdateTodo) -> anyhow::Result<Todo> {
        let mut store = self.write_store_ref();
        let todo = store.get(&id).context(RepositoryError::NotFound(id))?;
        let text = payload.text.unwrap_or(todo.text.clone());
        let completed = payload.completed.unwrap_or(todo.completed);
        let todo = Todo {
            id,
            text,
            completed,
        };
        store.insert(id, todo.clone());
        Ok(todo)
    }

    fn delete(&self, id: i32) -> anyhow::Result<()> {
        let mut store = self.write_store_ref();
        store.remove(&id).ok_or(RepositoryError::NotFound(id))?;
        Ok(())
    }
}

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

pub async fn create_todo<T: TodoRepository>(
    // 受取りの順番がによりエラーが出る場合がある
    // 参考記事：https://qiita.com/Sicut_study/items/5e5d6cce5ba48c225367
    Extension(repositories): Extension<Arc<T>>,
    Json(payload): Json<CreateTodo>
) -> impl IntoResponse {
    let todo = repositories.create(payload);

    (StatusCode::CREATED, Json(todo))
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
