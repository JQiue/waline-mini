use helpers::time::utc_now;
use sea_orm::{IntoActiveModel, Set};
use serde_json::{Value, json};

use crate::prelude::*;
use crate::{app::AppState, entities::wl_counter};

pub async fn get_article(
  state: &AppState,
  path: String,
  r#type: String,
) -> ServiceResult<Vec<Value>> {
  let mut data = vec![];
  if r#type == "time" {
    for path in path.split(',') {
      match state.repo.counter().get_counter(path).await? {
        Some(counter) => {
          data.push(json!({"time": counter.time}));
        }
        _ => {
          data.push(json!({"time": 0}));
        }
      }
    }
  } else if let Some(counter) = state.repo.counter().get_counter(&path).await? {
    data.push(json!({
      "reaction0": counter.reaction0,
      "reaction1": counter.reaction1,
      "reaction2": counter.reaction2,
      "reaction3": counter.reaction3,
      "reaction4": counter.reaction4,
      "reaction5": counter.reaction5,
    }));
  }
  Ok(data)
}

pub async fn update_article(
  state: &AppState,
  action: Option<String>,
  path: String,
  r#type: String,
) -> ServiceResult<Vec<wl_counter::Model>> {
  let mut data = vec![];
  if r#type == "time" {
    match state.repo.counter().get_counter(&path).await? {
      Some(counter) => {
        let time = counter.time.unwrap_or(0) + 1;
        let mut active_counter = counter.into_active_model();
        active_counter.time = Set(Some(time));
        active_counter.updated_at = Set(Some(utc_now()));
        data.push(state.repo.counter().update_counter(active_counter).await?)
      }
      _ => data.push(state.repo.counter().create_counter(path).await?),
    };
  } else {
    fn set_reaction_value(
      counter: wl_counter::Model,
      reaction: &str,
      action: Option<String>,
    ) -> wl_counter::ActiveModel {
      let mut active_counter = counter.clone().into_active_model();
      match reaction {
        "reaction0" => {
          active_counter.reaction0 = Set(Some(
            counter.reaction0.unwrap_or(0) + if action.is_none() { 1 } else { -1 },
          ));
        }
        "reaction1" => {
          active_counter.reaction1 = Set(Some(
            counter.reaction1.unwrap_or(0) + if action.is_none() { 1 } else { -1 },
          ));
        }
        "reaction2" => {
          active_counter.reaction2 = Set(Some(
            counter.reaction2.unwrap_or(0) + if action.is_none() { 1 } else { -1 },
          ));
        }
        "reaction3" => {
          active_counter.reaction3 = Set(Some(
            counter.reaction3.unwrap_or(0) + if action.is_none() { 1 } else { -1 },
          ));
        }
        "reaction4" => {
          active_counter.reaction4 = Set(Some(
            counter.reaction4.unwrap_or(0) + if action.is_none() { 1 } else { -1 },
          ));
        }
        "reaction5" => {
          active_counter.reaction5 = Set(Some(
            counter.reaction5.unwrap_or(0) + if action.is_none() { 1 } else { -1 },
          ));
        }
        "reaction6" => {
          active_counter.reaction6 = Set(Some(
            counter.reaction6.unwrap_or(0) + if action.is_none() { 1 } else { -1 },
          ));
        }
        "reaction7" => {
          active_counter.reaction7 = Set(Some(
            counter.reaction7.unwrap_or(0) + if action.is_none() { 1 } else { -1 },
          ));
        }
        "reaction8" => {
          active_counter.reaction8 = Set(Some(
            counter.reaction8.unwrap_or(0) + if action.is_none() { 1 } else { -1 },
          ));
        }
        _ => {}
      }
      active_counter
    }
    if let Some(counter) = state.repo.counter().get_counter(&path).await? {
      let mut active_counter = set_reaction_value(counter, &r#type, action);
      active_counter.updated_at = Set(Some(utc_now()));
      data.push(state.repo.counter().update_counter(active_counter).await?);
    }
  }
  Ok(data)
}
