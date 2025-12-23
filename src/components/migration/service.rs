use crate::components::migration::model::{CommentData, CreateDataBody, UpdateDataBody};

use crate::traits::LoggingResultErr;
use crate::types::ServiceResult;
use crate::{
  app::AppState,
  entities::{wl_comment, wl_counter, wl_users},
  prelude::AppError,
};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel, Set};
use serde_json::{Value, json};

use super::model::{CounterData, UserData};

pub async fn export_data(state: &AppState) -> ServiceResult<Value> {
  let comments = wl_comment::Entity::find()
    .into_partial_model::<CommentData>()
    .all(&state.repo.db)
    .await
    .log_err()?;
  let counters = wl_counter::Entity::find()
    .into_partial_model::<CounterData>()
    .all(&state.repo.db)
    .await
    .log_err()?;
  let users = wl_users::Entity::find()
    .into_partial_model::<UserData>()
    .all(&state.repo.db)
    .await
    .log_err()?;
  Ok(json!({
      "type": "waline",
      "version": 1,
      "time": Utc::now().timestamp_millis(),
      "tables": ["Comment", "Counter", "Users"],
      "data": {
        "Comment": comments,
        "Counter": counters,
        "Users": users,
      }
  }))
}

pub async fn create_comment_data(state: &AppState, body: CreateDataBody) -> ServiceResult<Value> {
  let CreateDataBody {
    comment,
    ip,
    link,
    mail,
    nick,
    status,
    ua,
    url,
    inserted_at,
    created_at,
    updated_at,
    ..
  } = body;

  let comment = wl_comment::ActiveModel {
    comment: Set(comment),
    inserted_at: Set(inserted_at),
    ip: Set(ip),
    link: Set(link),
    mail: Set(mail),
    nick: Set(nick),
    status: Set(status.unwrap()),
    ua: Set(ua),
    url: Set(url),
    created_at: Set(created_at),
    updated_at: Set(updated_at),
    ..Default::default()
  }
  .insert(&state.repo.db)
  .await?;
  Ok(json!({
    "objectId": comment.id,
    "comment": comment.comment,
    "ip": comment.ip,
    "link": comment.link,
    "mail": comment.mail,
    "nick": comment.nick,
    "status": comment.status,
    "ua": comment.ua,
    "url": comment.url,
    "insertedAt": comment.inserted_at,
    "createdAt": comment.created_at,
    "updatedAt": comment.updated_at,
  }))
}

pub async fn create_counter_data(
  state: &AppState,
  body: CreateDataBody,
) -> ServiceResult<wl_counter::Model> {
  let CreateDataBody {
    url,
    created_at,
    updated_at,
    time,
    reaction0,
    reaction1,
    reaction2,
    reaction3,
    reaction4,
    reaction5,
    reaction6,
    reaction7,
    reaction8,
    ..
  } = body;
  Ok(
    wl_counter::ActiveModel {
      time: Set(time),
      reaction0: Set(reaction0),
      reaction1: Set(reaction1),
      reaction2: Set(reaction2),
      reaction3: Set(reaction3),
      reaction4: Set(reaction4),
      reaction5: Set(reaction5),
      reaction6: Set(reaction6),
      reaction7: Set(reaction7),
      reaction8: Set(reaction8),
      url: Set(url.unwrap()),
      created_at: Set(created_at),
      updated_at: Set(updated_at),
      ..Default::default()
    }
    .insert(&state.repo.db)
    .await?,
  )
}

pub async fn update_comment_data(
  state: &AppState,
  object_id: u32,
  body: UpdateDataBody,
) -> ServiceResult<bool> {
  let UpdateDataBody { pid, rid, .. } = body;
  let mut active_comment = state
    .repo
    .comment()
    .get_comment(object_id)
    .await?
    .ok_or(AppError::Error)?
    .into_active_model();
  active_comment.pid = Set(pid);
  active_comment.rid = Set(rid);
  state.repo.comment().update_comment(active_comment).await?;
  Ok(true)
}

pub async fn create_user_data(state: &AppState, body: CreateDataBody) -> ServiceResult<bool> {
  let CreateDataBody {
    url,
    created_at,
    updated_at,
    two_factor_auth,
    display_name,
    email,
    label,
    password,
    r#type,
    ..
  } = body;

  let model = wl_users::ActiveModel {
    display_name: Set(display_name.unwrap()),
    email: Set(email.unwrap()),
    password: Set(password.unwrap()),
    user_type: Set(r#type.unwrap()),
    label: Set(label),
    url: Set(url),
    two_factor_auth: Set(two_factor_auth),
    created_at: Set(created_at),
    updated_at: Set(updated_at),
    ..Default::default()
  };
  match wl_users::Entity::insert(model).exec(&state.repo.db).await {
    Ok(_) => Ok(true),
    Err(err) => Err(err.into()),
  }
}

pub async fn update_user_data(state: &AppState, body: UpdateDataBody) -> Result<(), AppError> {
  let UpdateDataBody {
    two_factor_auth,
    display_name,
    email,
    label,
    password,
    r#type,
    url,
    created_at,
    updated_at,
    ..
  } = body;

  if state
    .repo
    .user()
    .has_user_by_email(&email.clone().unwrap_or("".to_owned()))
    .await?
  {
    let mut active_user = state
      .repo
      .user()
      .get_user_by_email(&email.clone().unwrap_or("".to_owned()))
      .await?
      .ok_or(AppError::UserNotFound)?
      .into_active_model();
    active_user.display_name = Set(display_name.unwrap());
    active_user.email = Set(email.unwrap());
    active_user.password = Set(password.unwrap());
    active_user.user_type = Set(r#type.unwrap());
    active_user.label = Set(label);
    active_user.url = Set(url);
    active_user.two_factor_auth = Set(two_factor_auth);
    active_user.created_at = Set(created_at);
    active_user.updated_at = Set(updated_at);
    match active_user.update(&state.repo.db).await.log_err() {
      Ok(_) => Ok(()),
      Err(_) => Err(AppError::Error),
    }
  } else {
    match (wl_users::ActiveModel {
      display_name: Set(display_name.unwrap()),
      email: Set(email.unwrap()),
      password: Set(password.unwrap()),
      user_type: Set(r#type.unwrap()),
      label: Set(label),
      url: Set(url),
      two_factor_auth: Set(two_factor_auth),
      created_at: Set(created_at),
      updated_at: Set(updated_at),
      ..Default::default()
    }
    .insert(&state.repo.db)
    .await
    .log_err())
    {
      Ok(_) => Ok(()),
      Err(_) => Err(AppError::Error),
    }
  }
}

pub async fn delete_data(state: &AppState, table: &str) -> Result<bool, AppError> {
  match table {
    "Comment" => {
      wl_comment::Entity::delete_many()
        .exec(&state.repo.db)
        .await?;
      Ok(true)
    }
    "Counter" => {
      wl_counter::Entity::delete_many()
        .exec(&state.repo.db)
        .await?;
      Ok(true)
    }
    "User" => Ok(true),
    _ => Err(AppError::Error),
  }
}
