use crate::repositories::{ CreateTodo, TodoRepository, UpdateTodo };
use axum::{ http::StatusCode, response::IntoResponse, extract::{ Extension, Path }, Json };
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

pub async fn find_todo<T: TodoRepository>(
    Path(id): Path<i32>,
    Extension(repository): Extension<Arc<T>>
) -> Result<impl IntoResponse, StatusCode> {
    let todo = repository.find(id).ok_or(StatusCode::NOT_FOUND)?;
    Ok((StatusCode::OK, Json(todo)))
}

pub async fn all_todo<T: TodoRepository>(Extension(
    repository,
): Extension<Arc<T>>) -> impl IntoResponse {
    // todoが1件も見つからないときは空配列のJsonを返す
    let todo = repository.all();
    (StatusCode::OK, Json(todo))
}

pub async fn update_todo<T: TodoRepository>(
    Path(id): Path<i32>,
    Extension(repository): Extension<Arc<T>>,
    Json(payload): Json<UpdateTodo>
) -> Result<impl IntoResponse, StatusCode> {
    let todo = repository.update(id, payload).or(Err(StatusCode::NOT_FOUND))?;
    Ok((StatusCode::CREATED, Json(todo)))
}

pub async fn delete_todo<T: TodoRepository>(
    Path(id): Path<i32>,
    Extension(repository): Extension<Arc<T>>
) -> StatusCode {
    repository
        .delete(id)
        .map(|_| StatusCode::NO_CONTENT)
        .unwrap_or(StatusCode::NOT_FOUND)
}
