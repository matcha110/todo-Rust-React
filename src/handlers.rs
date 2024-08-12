use crate::repositories::{ CreateTodo, TodoRepository };
use axum::{ http::StatusCode, response::IntoResponse, extract::Extension, Json };
use std::sync::Arc;

pub async fn create_todo<T: TodoRepository>(
    // 受取りの順番がによりエラーが出る場合がある
    // 参考記事：https://qiita.com/Sicut_study/items/5e5d6cce5ba48c225367
    Extension(repositories): Extension<Arc<T>>,
    Json(payload): Json<CreateTodo>
) -> impl IntoResponse {
    let todo = repositories.create(payload);

    (StatusCode::CREATED, Json(todo))
}
