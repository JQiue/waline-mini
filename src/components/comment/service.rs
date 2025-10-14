use actix_web::rt::spawn;
use helpers::{
  jwt,
  time::{self, utc_now},
};
use instant_akismet::CheckResult;
use sea_orm::{ItemsAndPagesNumber, Set};
use serde_json::{Value, json};

use crate::{
  app::AppState,
  components::comment::model::*,
  entities::wl_comment,
  helpers::{
    avatar::get_avatar,
    email::{Notification, NotifyType, send_email_notification},
    markdown::render_md_to_html,
    spam::check_comment,
    ua,
  },
  prelude::AppError,
  types::ServiceResult,
};

pub async fn get_comment_info(
  state: &AppState,
  path: String,
  page: i32,
  page_size: i32,
  sort_by: String,
  token: Result<String, AppError>,
) -> ServiceResult<Value> {
  if let Some(result) = state.comment_cache.lock().unwrap().get(path.clone(), page) {
    return Ok(result);
  }

  let mut is_admin = false;
  if let Ok(token) = token {
    if let Ok(email) = jwt::verify::<String>(&token, &state.jwt_token).map(|t| t.claims.data) {
      if state
        .repo
        .user()
        .is_admin_user(&email)
        .await
        .unwrap_or(false)
      {
        is_admin = true;
      }
    }
  }
  let (
    ItemsAndPagesNumber {
      number_of_items,
      number_of_pages,
    },
    parrent_comments,
  ) = state
    .repo
    .comment()
    .get_root_comment(&path, sort_by, page as u64, page_size as u64, is_admin)
    .await?;
  // Get comment count for articles
  let mut count = number_of_items;
  let total_pages = number_of_pages;
  let levels = state.levels.as_ref();
  let mut comment_data = vec![];

  for parrent_comment in parrent_comments {
    let c = state
      .repo
      .comment()
      .get_comment_count_by_nick_and_mail(
        parrent_comment.nick.clone(),
        parrent_comment.mail.clone(),
      )
      .await?;
    let level = levels.map(|levels| get_level(c as usize, levels));
    let mut parrent_data = build_data_entry(
      parrent_comment.clone(),
      level,
      &state.ip2region,
      state.disable_useragent,
      state.disable_region,
    );

    if let Some(user_id) = parrent_data.user_id {
      if let Some(user) = state.repo.user().get_user_by_id(user_id as u32).await? {
        parrent_data.label = user.label;
        parrent_data.r#type = Some(user.user_type);
        if let Some(avatar) = user.avatar {
          parrent_data.avatar = avatar;
        }
      }
    } else {
      parrent_data.avatar =
        get_avatar(&parrent_comment.mail.clone().unwrap_or("default".to_owned()));
    }

    if is_admin {
      parrent_data.mail = parrent_comment.mail.clone();
      parrent_data.ip = parrent_comment.ip.clone();
    }

    let subcomments = state
      .repo
      .comment()
      .get_subcomments(&path, parrent_comment.id, is_admin)
      .await?;
    count += subcomments.len() as u64;

    for subcomment in subcomments {
      let c = state
        .repo
        .comment()
        .get_comment_count_by_nick_and_mail(
          parrent_comment.nick.clone(),
          parrent_comment.mail.clone(),
        )
        .await?;
      let level = levels.map(|levels| get_level(c as usize, levels));
      let mut subcomment_data = build_data_entry(
        subcomment.clone(),
        level,
        &state.ip2region,
        state.disable_useragent,
        state.disable_region,
      );

      if let Some(user_id) = subcomment_data.user_id {
        let user = state.repo.user().get_user_by_id(user_id as u32).await?;
        if let Some(user) = user {
          subcomment_data.label = user.label;
          subcomment_data.r#type = Some(user.user_type);
          if let Some(avatar) = user.avatar {
            parrent_data.avatar = avatar;
          }
        }
      } else {
        parrent_data.avatar =
          get_avatar(&parrent_comment.mail.clone().unwrap_or("default".to_owned()));
      }

      if is_admin {
        subcomment_data.mail = subcomment_data.mail.clone();
        subcomment_data.ip = subcomment_data.ip.clone();
      }
      subcomment_data.reply_user = Some(json!({
        "avatar": get_avatar(&parrent_comment.mail.clone().unwrap_or("default".to_owned())),
        "link": parrent_comment.link,
        "nick": parrent_comment.nick,
      }));
      parrent_data.children.push(subcomment_data)
    }
    comment_data.push(parrent_data)
  }

  let data = json!({
    "count": count,
    "data": comment_data,
    "page": page,
    "pageSize": page_size,
    "totalPages": total_pages
  });
  state
    .comment_cache
    .lock()
    .unwrap()
    .insert(path, page, data.clone());
  Ok(data)
}

pub async fn get_comment_info_by_admin(
  state: &AppState,
  owner: String,
  email: String,
  keyword: String,
  status: String,
  page: i32,
) -> ServiceResult<Value> {
  let (
    ItemsAndPagesNumber {
      number_of_items: _,
      number_of_pages,
    },
    comments,
  ) = state
    .repo
    .comment()
    .get_comments_list_by_admin(&email, &status, &keyword, page as u64, 10, owner)
    .await?;
  let mut data = vec![];

  for comment in comments.iter() {
    let mut data_entry = build_data_entry(
      comment.clone(),
      None,
      &state.ip2region,
      state.disable_useragent,
      state.disable_region,
    );
    if let Some(user_id) = data_entry.user_id {
      if let Some(user) = state.repo.user().get_user_by_id(user_id as u32).await? {
        data_entry.label = user.label;
        data_entry.r#type = Some(user.user_type);
      }
    }
    data.push(data_entry);
  }
  Ok(json!({
    "data": data,
    "page": page,
    "pageSize": 10,
    "spamCount": 0,
    "totalPages": number_of_pages,
    "waitingCount": 0,
  }))
}

pub async fn create_comment<'a>(
  state: &AppState,
  comment: String,
  link: Option<String>,
  mail: String,
  nick: String,
  ua: String,
  url: String,
  pid: Option<i32>,
  rid: Option<i32>,
  _at: Option<String>,
  ip: String,
  user_type: UserType,
  lang: String,
) -> Result<Value, AppError> {
  state.comment_cache.lock().unwrap().invalidate(&url);
  let html_output = render_md_to_html(&comment);
  let mut avatar = get_avatar("");
  let mut new_comment = create_comment_model(
    None,
    comment.clone(),
    link,
    mail.clone(),
    nick.clone(),
    ua.clone(),
    url.clone(),
    ip.clone(),
    pid,
    rid,
  );
  let (browser, os) = ua::parse(ua);
  let mut data = json!({
    "addr":"",
    "browser": browser,
    "os": os,
    "comment": html_output,
  });
  match user_type {
    UserType::Anonymous => {
      new_comment.status = Set(if state.comment_audit {
        "waiting".to_string()
      } else if has_forbidden_word(&comment, &state.forbidden_words) {
        "spam".to_string()
      } else if matches!(
        check_comment(nick, mail, ip, comment).await?,
        CheckResult::Ham
      ) {
        "approved".to_string()
      } else {
        "spam".to_string()
      });
    }
    UserType::Guest(email) => {
      if let Some(user) = state.repo.user().get_user_by_email(&email).await? {
        new_comment.user_id = Set(Some(user.id as i32));
        new_comment.status = Set(if state.comment_audit {
          "waiting".to_string()
        } else if has_forbidden_word(&comment, &state.forbidden_words) {
          "spam".to_string()
        } else if matches!(
          check_comment(nick, mail, ip, comment).await?,
          CheckResult::Ham
        ) {
          "approved".to_string()
        } else {
          "spam".to_string()
        });
        data["label"] = json!(user.label);
        data["mail"] = json!(user.email);
        data["type"] = json!(user.user_type);
        data["user_id"] = json!(user.id);
        if let Some(user_avatar) = user.avatar {
          avatar = user_avatar;
        } else {
          avatar = get_avatar(&user.email);
        }
      }
    }
    UserType::Administrator(email) => {
      if let Some(user) = state.repo.user().get_user_by_email(&email).await? {
        new_comment.user_id = Set(Some(user.id as i32));
        new_comment.status = Set("approved".to_string());
        data["label"] = json!(user.label);
        data["mail"] = json!(user.email);
        data["type"] = json!(user.user_type);
        data["user_id"] = json!(user.id);
        if let Some(user_avatar) = user.avatar {
          avatar = user_avatar;
        } else {
          avatar = get_avatar(&user.email);
        }
      }
    }
  }
  let comment = state.repo.comment().create_comment(new_comment).await?;
  data["avatar"] = json!(avatar);
  data["like"] = json!(comment.like);
  data["ip"] = json!(comment.ip);
  data["link"] = json!(comment.link);
  data["nick"] = json!(comment.nick);
  data["objectId"] = json!(comment.id);
  data["orig"] = json!(comment.comment);
  data["status"] = json!(comment.status);
  data["time"] = json!(comment.created_at.unwrap_or(utc_now()).timestamp_millis());
  data["url"] = json!(comment.url);
  if let Some(pid) = pid {
    data["pid"] = json!(pid);
  }
  if let Some(rid) = rid {
    data["rid"] = json!(rid);
  };
  spawn(async move {
    send_email_notification(Notification {
      sender_name: comment.nick.unwrap(),
      sender_email: comment.mail.unwrap(),
      comment_id: comment.id,
      comment: comment.comment.unwrap(),
      url: comment.url.unwrap(),
      notify_type: NotifyType::NewComment,
      lang: Some(&lang),
    });
  });
  Ok(data)
}

pub async fn delete_comment(state: &AppState, id: u32, token: String) -> ServiceResult<()> {
  let email = jwt::verify::<String>(&token, &state.jwt_token)?.claims.data;
  let user = state
    .repo
    .user()
    .get_user_by_email(&email)
    .await?
    .ok_or(AppError::UserNotFound)?;
  let pass = if user.user_type == "administrator" {
    true
  } else {
    state.repo.comment().is_comment_owner(id, user.id).await?
  };
  if !pass {
    return Err(AppError::Forbidden);
  }
  state.repo.comment().delete_comment(id).await?;
  state.comment_cache.lock().unwrap().clear();
  Ok(())
}

pub async fn update_comment(
  state: &AppState,
  email: String,
  id: u32,
  status: Option<String>,
  like: Option<bool>,
  comment: Option<String>,
  link: Option<String>,
  mail: Option<String>,
  nick: Option<String>,
  ua: Option<String>,
  url: Option<String>,
  sticky: Option<i8>,
) -> ServiceResult<Value> {
  let mut active_comment = wl_comment::ActiveModel {
    id: Set(id),
    updated_at: Set(Some(time::utc_now())),
    ..Default::default()
  };

  let user = state
    .repo
    .user()
    .get_user_by_email(&email)
    .await?
    .ok_or(AppError::UserNotFound)?;

  if !state.repo.comment().is_comment_owner(id, user.id).await? && user.user_type != "administrator"
  {
    return Err(AppError::Forbidden);
  }

  if let Some(like) = like {
    let comment = state
      .repo
      .comment()
      .get_comment(id)
      .await?
      .ok_or(AppError::Error)?;
    active_comment.like = Set(Some(comment.like.unwrap_or(0) + if like { 1 } else { -1 }));
  }

  if let Some(status) = status {
    active_comment.status = Set(status);
  }

  if let Some(sticky) = sticky {
    active_comment.sticky = Set(Some(sticky));
  }

  if let Some(comment) = comment {
    active_comment.comment = Set(Some(comment));
  }

  if let Some(ua) = ua {
    active_comment.ua = Set(Some(ua));
  }

  if let Some(nick) = nick {
    active_comment.nick = Set(Some(nick));
  }

  if let Some(link) = link {
    active_comment.link = Set(Some(link));
  }

  if let Some(mail) = mail {
    active_comment.mail = Set(Some(mail));
  }

  if let Some(url) = url {
    active_comment.url = Set(Some(url));
  }

  let updated_comment = state.repo.comment().update_comment(active_comment).await?;
  state.comment_cache.lock().unwrap().clear();
  let (browser, os) = ua::parse(updated_comment.ua.unwrap_or("".to_owned()));
  let like = updated_comment.like.unwrap_or(0);
  let time = updated_comment.created_at.unwrap().timestamp_millis();
  let pid = updated_comment.pid;
  let rid = updated_comment.rid;
  let html_output = render_md_to_html(updated_comment.comment.clone().unwrap().as_str());
  if state.repo.comment().is_anonymous(id).await? {
    let data = json!({
      "addr":"",
      "avatar": get_avatar(""),
      "browser": browser,
      "comment": html_output,
      "ip": updated_comment.ip,
      "mail": updated_comment.mail,
      "user_id": updated_comment.user_id,
      "like": like,
      "link": updated_comment.link,
      "nick": updated_comment.nick,
      "objectId": updated_comment.id,
      "orig": updated_comment.comment,
      "os": os,
      "status": updated_comment.status,
      "time": time,
      "url": updated_comment.url,
    });
    Ok(data)
  } else {
    let user = state
      .repo
      .user()
      .get_user_by_id(updated_comment.user_id.unwrap() as u32)
      .await?
      .ok_or(AppError::UserNotFound)?;
    let mut data = json!({
      "addr":"",
      "avatar": get_avatar(&user.email),
      "browser": browser,
      "comment": html_output,
      "ip": updated_comment.ip,
      "label": user.label,
      "mail": user.email.clone(),
      "type": user.user_type,
      "user_id": updated_comment.user_id,
      "like": like,
      "link": updated_comment.link,
      "nick": updated_comment.nick,
      "objectId": updated_comment.id,
      "orig": updated_comment.comment,
      "os": os,
      "status": updated_comment.status,
      "time": time,
      "url": updated_comment.url,
    });
    if let Some(pid) = pid {
      data["pid"] = json!(pid);
    }
    if let Some(rid) = rid {
      data["rid"] = json!(rid);
    };
    Ok(data)
  }
}
