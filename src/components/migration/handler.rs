use actix_web::{
  HttpResponse, delete, get, post, put,
  web::{Data, Json, Query},
};

use crate::{
  app::AppState,
  components::migration::{model::*, service},
  prelude::{AppError, Response},
  traits::IntoHttpResponse,
};

#[get("/db")]
pub async fn export_data(
  state: Data<AppState>,
  query: Query<ExportQuery>,
) -> Result<HttpResponse, AppError> {
  service::export_data(&state)
    .await
    .into_http_response(Some(&query.lang))
}

#[post("/db")]
pub async fn create_data(
  state: Data<AppState>,
  query: Query<CreateDataQuery>,
  body: Json<CreateDataBody>,
) -> Result<HttpResponse, AppError> {
  let Query(CreateDataQuery { table, lang }) = query;
  match table.as_str() {
    "Comment" => service::create_comment_data(&state, body.0)
      .await
      .into_http_response(Some(&lang)),
    "Counter" => service::create_counter_data(&state, body.0)
      .await
      .into_http_response(Some(&lang)),
    "Users" => service::create_user_data(&state, body.0)
      .await
      .into_http_response(Some(&lang)),
    _ => Response::<()>::new_error(AppError::Error, Some(&lang)),
  }
}

#[put("/db")]
pub async fn update_data(
  state: Data<AppState>,
  query: Query<UpdateDataQuery>,
  body: Json<UpdateDataBody>,
) -> Result<HttpResponse, AppError> {
  match query.table.as_str() {
    "Comment" => service::update_comment_data(&state, query.object_id, body.0)
      .await
      .into_http_response(Some(&query.lang)),
    "Users" => service::update_user_data(&state, body.0)
      .await
      .into_http_response(Some(&query.lang)),
    _ => Response::<()>::new_error(AppError::Error, Some(&query.lang)),
  }
}

#[delete("/db")]
pub async fn delete_data(
  state: Data<AppState>,
  query: Query<DeleteQuery>,
) -> Result<HttpResponse, AppError> {
  let Query(DeleteQuery { table, lang }) = query;
  service::delete_data(&state, &table)
    .await
    .into_http_response(Some(&lang))
}
