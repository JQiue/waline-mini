use actix_web::web::ServiceConfig;
use shuttle_actix_web::ShuttleActixWeb;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{EnvFilter, filter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

mod app;
mod components;
mod config;
mod entities;
mod error;
mod helpers;
mod locales;
mod prelude;
mod repository;
mod response;
mod traits;

#[shuttle_runtime::main]
async fn main() -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
  Ok(app::start().await.into())
}
