use helpers::time::utc_now;
use sea_orm::{IntoActiveModel, Set};
use serde_json::{Value, json};

use crate::prelude::*;
use crate::{app::AppState, entities::wl_counter};

pub async fn get_article(
  state: &AppState,
  path: String,
  r#type: String,
) -> Result<Vec<Value>, AppError> {
  let mut data = vec![];
  if r#type == "time" {
    for path in path.split(',') {
      match state.repo.counter().get_counter(path).await? { Some(counter) => {
        data.push(json!({"time": counter.time}));
      } _ => {
        data.push(json!({"time": 0}));
      }}
    }
  } else { match state.repo.counter().get_counter(&path).await? { Some(counter) => {
    data.push(json!({
      "reaction0": counter.reaction0,
      "reaction1": counter.reaction1,
      "reaction2": counter.reaction2,
      "reaction3": counter.reaction3,
      "reaction4": counter.reaction4,
      "reaction5": counter.reaction5,
    }));
  } _ => {}}}
  Ok(data)
}

pub async fn update_article(
  state: &AppState,
  action: Option<String>,
  path: String,
  r#type: String,
) -> Result<Vec<wl_counter::Model>, AppError> {
  let mut data = vec![];
  if r#type == "time" {
    match state.repo.counter().get_counter(&path).await? { Some(counter) => {
      let time = counter.time.unwrap_or(0) + 1;
      let mut active_counter = counter.into_active_model();
      active_counter.time = Set(Some(time));
      active_counter.updated_at = Set(Some(utc_now()));
      data.push(state.repo.counter().update_counter(active_counter).await?)
    } _ => {
      data.push(state.repo.counter().create_counter(path).await?)
    }};
  } else {
    fn set_reaction_value(
      mut counter: wl_counter::Model,
      reaction: &str,
      action: Option<String>,
    ) -> wl_counter::Model {
      match reaction {
        "reaction0" => {
          counter.reaction0 = if action.is_none() {
            Some(counter.reaction0.unwrap_or(0) + 1)
          } else {
            Some(counter.reaction0.unwrap_or(1) - 1)
          }
        }
        "reaction1" => {
          counter.reaction1 = if action.is_none() {
            Some(counter.reaction1.unwrap_or(0) + 1)
          } else {
            Some(counter.reaction1.unwrap_or(1) - 1)
          }
        }
        "reaction2" => {
          counter.reaction2 = if action.is_none() {
            Some(counter.reaction2.unwrap_or(0) + 1)
          } else {
            Some(counter.reaction2.unwrap_or(1) - 1)
          }
        }
        "reaction3" => {
          counter.reaction3 = if action.is_none() {
            Some(counter.reaction3.unwrap_or(0) + 1)
          } else {
            Some(counter.reaction3.unwrap_or(1) - 1)
          }
        }
        "reaction4" => {
          counter.reaction4 = if action.is_none() {
            Some(counter.reaction4.unwrap_or(0) + 1)
          } else {
            Some(counter.reaction4.unwrap_or(1) - 1)
          }
        }
        "reaction5" => {
          counter.reaction5 = if action.is_none() {
            Some(counter.reaction5.unwrap_or(0) + 1)
          } else {
            Some(counter.reaction5.unwrap_or(1) - 1)
          }
        }
        "reaction6" => {
          counter.reaction6 = if action.is_none() {
            Some(counter.reaction6.unwrap_or(0) + 1)
          } else {
            Some(counter.reaction6.unwrap_or(1) - 1)
          }
        }
        "reaction7" => {
          counter.reaction7 = if action.is_none() {
            Some(counter.reaction7.unwrap_or(0) + 1)
          } else {
            Some(counter.reaction7.unwrap_or(1) - 1)
          }
        }
        "reaction8" => {
          counter.reaction8 = if action.is_none() {
            Some(counter.reaction8.unwrap_or(0) + 1)
          } else {
            Some(counter.reaction8.unwrap_or(1) - 1)
          }
        }
        _ => {}
      }
      counter
    }
    if let Some(counter) = state.repo.counter().get_counter(&path).await? {
      let active_counter = set_reaction_value(counter, &r#type, action).into_active_model();
      data.push(state.repo.counter().update_counter(active_counter).await?);
    }
  }
  Ok(data)
}
