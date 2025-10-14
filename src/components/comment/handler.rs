use actix_web::{
  HttpRequest, HttpResponse, delete, get, post, put,
  web::{Data, Json, Path, Query},
};
use helpers::jwt;

use crate::{
  app::AppState,
  components::comment::{model::*, service},
  config::EnvConfig,
  helpers::header::{extract_ip, extract_token},
  prelude::{AppError, Response},
  traits::IntoHttpResponse,
};

#[get("/comment")]
async fn get_comment_info(
  req: HttpRequest,
  state: Data<AppState>,
  query: Query<GetCommentQuery>,
) -> Result<HttpResponse, AppError> {
  let Query(GetCommentQuery {
    lang,
    path,
    page_size,
    page,
    sort_by,
    r#type: _,
    owner,
    status,
    keyword,
  }) = query.clone();
  if let Some(path) = path {
    let fields = query.validate_by_path();
    if fields.is_err() {
      return Response::<()>::new_error(AppError::Error, Some(&lang));
    }
    let token = extract_token(&req);
    service::get_comment_info(
      &state,
      path,
      page,
      page_size.unwrap(),
      sort_by.unwrap(),
      token,
    )
    .await
    .into_http_response(Some(&lang))
  } else {
    let fields = query.validate_by_admin();
    if fields.is_err() {
      return Response::<()>::new_error(AppError::Error, Some(&lang));
    }
    let token = extract_token(&req).unwrap();
    let email = match jwt::verify::<String>(&token, &state.jwt_token) {
      Ok(token_data) => token_data.claims.data,
      Err(err) => return Response::<()>::new_error(err.into(), Some(&lang)),
    };
    let is = state.repo.user().is_admin_user(&email).await?;
    if !is {
      return Response::<()>::new_error(AppError::Unauthorized, Some(&lang));
    }
    service::get_comment_info_by_admin(
      &state,
      owner.unwrap(),
      email,
      keyword.unwrap(),
      status.unwrap(),
      page,
    )
    .await
    .into_http_response(Some(&lang))
  }
}

#[post("/comment")]
async fn create_comment(
  req: HttpRequest,
  state: Data<AppState>,
  query: Query<CreateCommentQuery>,
  body: Json<CreateCommentBody>,
) -> HttpResponse {
  let Query(CreateCommentQuery { lang }) = query;
  let Json(CreateCommentBody {
    comment,
    link,
    mail,
    nick,
    ua,
    url,
    pid,
    rid,
    at,
  }) = body;
  let mut user_type = UserType::Anonymous;
  let mut is_admin = false;
  let client_ip = extract_ip(&req);
  let pass = match extract_token(&req) {
    Ok(token) => match jwt::verify::<String>(&token, &state.jwt_token) {
      Ok(verified_token) => {
        if state
          .repo
          .user()
          .is_admin_user(&verified_token.claims.data)
          .await
          .unwrap()
        {
          is_admin = true;
          user_type = UserType::Administrator(verified_token.claims.data);
          true
        } else {
          user_type = UserType::Guest(verified_token.claims.data);
          state.rate_limiter.check_and_update(&client_ip, 1)
        }
      }
      Err(err) => {
        tracing::error!("{}", err);
        return HttpResponse::Ok().json(Response::<()>::error(AppError::Unauthorized, Some(&lang)));
      }
    },
    _ => {
      if &state.login == "force" {
        return HttpResponse::Ok().json(Response::<()>::error(AppError::Unauthorized, Some(&lang)));
      }
      state.rate_limiter.check_and_update(&client_ip, 1)
    }
  };
  if !pass {
    return HttpResponse::Ok().json(Response::<()>::error(
      AppError::FrequencyLimited,
      Some(&lang),
    ));
  }
  if !is_admin {
    let EnvConfig {
      disallow_ip_list, ..
    } = EnvConfig::load_env().unwrap();
    if disallow_ip_list.contains(&client_ip) {
      tracing::info!("Comment IP {client_ip} is in disallowIPList");
      return HttpResponse::Ok().json(Response::<()>::error(AppError::Forbidden, Some(&lang)));
    }
  }
  if state
    .repo
    .comment()
    .is_duplicate(&url, &mail, &nick, link.as_deref(), &comment)
    .await
    .unwrap()
    && !is_admin
  {
    return HttpResponse::Ok().json(Response::<()>::error(
      AppError::DuplicateContent,
      Some(&lang),
    ));
  }
  match service::create_comment(
    &state,
    comment,
    link,
    mail,
    nick,
    ua,
    url,
    pid,
    rid,
    at,
    client_ip,
    user_type,
    lang.clone(),
  )
  .await
  {
    Ok(data) => HttpResponse::Ok().json(Response::success(Some(data))),
    Err(err) => HttpResponse::Ok().json(Response::<()>::error(err, Some(&lang))),
  }
}

#[delete("/comment/{id}")]
pub async fn delete_comment(
  req: HttpRequest,
  state: Data<AppState>,
  path: Path<u32>,
) -> Result<HttpResponse, AppError> {
  let id = path.into_inner();
  let token = extract_token(&req)?;
  match service::delete_comment(&state, id, token).await {
    Ok(_) => Response::<()>::new_success(None),
    Err(err) => Response::<()>::new_error(err, None),
  }
}

#[put("/comment/{id}")]
async fn update_comment(
  req: HttpRequest,
  state: Data<AppState>,
  path: Path<u32>,
  body: Json<UpdateCommentBody>,
) -> HttpResponse {
  let actix_web::web::Json(UpdateCommentBody {
    status,
    like,
    comment,
    link,
    mail,
    nick,
    ua,
    url,
    sticky,
  }) = body;
  let id: u32 = path.into_inner();
  if like.is_some() {
    match service::update_comment(
      &state,
      String::new(),
      id,
      status,
      like,
      comment,
      link,
      mail,
      nick,
      ua,
      url,
      sticky,
    )
    .await
    {
      Ok(data) => return HttpResponse::Ok().json(Response::success(Some(data))),
      Err(err) => return HttpResponse::Ok().json(Response::<()>::error(err, None)),
    }
  }
  match extract_token(&req) {
    Ok(token) => match jwt::verify::<String>(&token, &state.jwt_token) {
      Ok(data) => match service::update_comment(
        &state,
        data.claims.data,
        id,
        status,
        like,
        comment,
        link,
        mail,
        nick,
        ua,
        url,
        sticky,
      )
      .await
      {
        Ok(data) => HttpResponse::Ok().json(Response::success(Some(data))),
        Err(err) => HttpResponse::Ok().json(Response::<()>::error(err, None)),
      },
      Err(_) => HttpResponse::Ok().json(Response::<()>::error(AppError::Unauthorized, None)),
    },
    _ => HttpResponse::Ok().json(Response::<()>::error(AppError::Unauthorized, None)),
  }
}
