use actix_web::{
  HttpRequest, HttpResponse, delete, get,
  http::{self, header::AUTHORIZATION},
  post, put,
  web::{Data, Json, Path, Query},
};

use crate::{
  app::AppState,
  components::user::{model::*, service},
  helpers::header::{extract_origin, extract_token, extract_token_from_header},
  prelude::{AppError, Response},
};

#[post("/user")]
pub async fn user_register(
  req: HttpRequest,
  state: Data<AppState>,
  query: Query<UserRegisterQuery>,
  body: Json<UserRegisterBody>,
) -> HttpResponse {
  let Query(UserRegisterQuery { lang }) = query;
  let Json(UserRegisterBody {
    display_name,
    email,
    password,
    url,
  }) = body;
  match service::user_register(
    &state,
    display_name,
    email,
    password,
    url,
    req
      .headers()
      .get("host")
      .unwrap()
      .to_str()
      .unwrap()
      .to_string(),
    &lang,
  )
  .await
  {
    Ok(data) => HttpResponse::Ok().json(Response::success(Some(data))),
    Err(err) => HttpResponse::Ok().json(Response::<()>::error(err, Some(&lang))),
  }
}

#[post("/token")]
pub async fn user_login(state: Data<AppState>, body: Json<UserLoginBody>) -> HttpResponse {
  let Json(UserLoginBody {
    code,
    email,
    password,
  }) = body;
  match service::user_login(&state, code, email, password).await {
    Ok(data) => HttpResponse::Ok().json(Response::success(Some(data))),
    Err(err) => HttpResponse::Ok().json(Response::<()>::error(err, None)),
  }
}

#[delete("/token")]
pub async fn user_logout() -> HttpResponse {
  HttpResponse::Ok().json(Response::<()>::success(None))
}

#[get("/token")]
async fn get_login_user_info(req: HttpRequest, state: Data<AppState>) -> HttpResponse {
  match extract_token_from_header(&req.headers().get(AUTHORIZATION)) { Some(token) => {
    match service::get_login_user_info(&state, token).await {
      Ok(data) => HttpResponse::Ok().json(Response::success(Some(data))),
      Err(err) => HttpResponse::Ok().json(Response::<()>::error(err, None)),
    }
  } _ => {
    HttpResponse::Ok().json(Response::<()>::error(AppError::Unauthorized, None))
  }}
}

#[put("/user")]
pub async fn set_user_profile(
  req: HttpRequest,
  state: Data<AppState>,
  body: Json<SetUserProfileBody>,
) -> HttpResponse {
  let Json(SetUserProfileBody {
    display_name,
    label,
    url,
    password,
    avatar,
    two_factor_auth,
  }) = body;
  match extract_token(&req) {
    Ok(token) => {
      match service::set_user_profile(
        &state,
        token,
        display_name,
        label,
        url,
        password,
        avatar,
        two_factor_auth,
      )
      .await
      {
        Ok(_) => HttpResponse::Ok().json(Response::<()>::success(None)),
        Err(err) => HttpResponse::Ok().json(Response::<()>::error(err, None)),
      }
    }
    Err(err) => HttpResponse::Ok().json(Response::<()>::error(err, None)),
  }
}

// WARNING
#[put("/user/{user_id}")]
pub async fn set_user_type(
  req: HttpRequest,
  state: Data<AppState>,
  path: Path<u32>,
  body: Json<SetUserTypeBody>,
) -> HttpResponse {
  let user_id = path.into_inner();
  let Json(SetUserTypeBody { r#type }) = body;
  match extract_token(&req) {
    Ok(token) => match service::set_user_type(&state, token, user_id, r#type).await {
      Ok(_) => HttpResponse::Ok().json(Response::<()>::success(None)),
      Err(err) => HttpResponse::Ok().json(Response::<()>::error(err, None)),
    },
    Err(_) => HttpResponse::Ok().json(Response::<()>::error(AppError::Unauthorized, None)),
  }
}

#[get("/user")]
pub async fn get_user_info(state: Data<AppState>, query: Query<GetUserQuery>) -> HttpResponse {
  let Query(GetUserQuery { email, lang, page }) = query;
  if let Some(page) = page {
    match service::get_user_info_list(&state, page).await {
      Ok(data) => HttpResponse::Ok().json(Response::success(Some(data))),
      Err(err) => HttpResponse::Ok().json(Response::<()>::error(err, Some(&lang))),
    }
  } else {
    match service::get_user_info(&state, email).await {
      Ok(data) => HttpResponse::Ok().json(Response::success(Some(data))),
      Err(_) => HttpResponse::Ok().json(Response::<()>::success(None)),
    }
  }
}

#[get("/verification")]
pub async fn verification(state: Data<AppState>, query: Query<VerificationQuery>) -> HttpResponse {
  let Query(VerificationQuery { email, token }) = query;
  match service::verification(&state, email, token).await {
    Ok(_) => HttpResponse::Found()
      .append_header((http::header::LOCATION, "/ui/login"))
      .finish(),
    Err(err) => HttpResponse::Ok().json(Response::<()>::error(err, None)),
  }
}

#[post("/token/2fa")]
pub async fn set_2fa(
  req: HttpRequest,
  state: Data<AppState>,
  body: Json<Set2faBody>,
) -> HttpResponse {
  let Json(Set2faBody { code, secret }) = body;
  match extract_token(&req) {
    Ok(token) => match service::set_2fa(&state, token, code, secret).await {
      Ok(data) => HttpResponse::Ok().json(Response::success(Some(data))),
      Err(_) => HttpResponse::Ok().json(Response::<()>::error(AppError::Unauthorized, None)),
    },
    Err(_) => HttpResponse::Ok().json(Response::<()>::error(AppError::Unauthorized, None)),
  }
}

#[get("/token/2fa")]
pub async fn get_2fa(
  req: HttpRequest,
  state: Data<AppState>,
  query: Query<Get2faQuery>,
) -> HttpResponse {
  let Query(Get2faQuery { lang, email }) = query;
  let token = extract_token(&req).map_or(None, Some);
  match service::get_2fa(&state, token, email).await {
    Ok(data) => HttpResponse::Ok().json(Response::success(Some(data))),
    Err(err) => HttpResponse::Ok().json(Response::<()>::error(err, Some(&lang))),
  }
}

#[put("/user/password")]
pub async fn modify_password(
  req: HttpRequest,
  state: Data<AppState>,
  query: Query<UserPasswordQuery>,
  body: Json<UserPasswordBody>,
) -> HttpResponse {
  let Query(UserPasswordQuery { lang }) = query;
  let Json(UserPasswordBody { email }) = body;
  let origin = extract_origin(&req);
  match service::modify_password(&state, email, &origin, &lang).await {
    Ok(data) => HttpResponse::Ok().json(Response::success(Some(data))),
    Err(err) => HttpResponse::Ok().json(Response::<()>::error(err, Some(&lang))),
  }
}
