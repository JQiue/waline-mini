use crate::prelude::*;
use crate::{
  app::AppState,
  components::article::{model::*, service},
};

use actix_web::{
  HttpResponse, get, post,
  web::{Data, Json, Query},
};

#[get("/article")]
async fn get_article(
  data: Data<AppState>,
  query: Query<GetArticleQuery>,
) -> Result<HttpResponse, AppError> {
  let Query(GetArticleQuery { path, r#type, lang }) = query;
  service::get_article(&data, path, r#type)
    .await
    .into_http_response(Some(&lang))
}

#[post("/article")]
async fn update_article(
  data: Data<AppState>,
  query: Query<UpdateArticleQuery>,
  body: Json<UpdateArticleBody>,
) -> Result<HttpResponse, AppError> {
  let Json(UpdateArticleBody {
    action,
    path,
    r#type,
  }) = body;
  let Query(UpdateArticleQuery { lang }) = query;
  service::update_article(&data, action, path, r#type)
    .await
    .into_http_response(Some(&lang))
}
