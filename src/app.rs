//! app
use std::{
  collections::HashMap,
  sync::{Arc, Mutex},
  time::{Duration, Instant},
};

use crate::{
  components::{
    article,
    comment::{self},
    migration,
    ui::{self, handler::ui_page},
    user,
  },
  config::EnvConfig,
  error::AppError,
  helpers::ip::Ip2Region,
  repository::RepositoryManager,
};

use actix_cors::Cors;
use actix_web::{
  middleware,
  web::{self, ServiceConfig},
  HttpResponse,
};
use sea_orm::Database;
use serde_json::Value;
use tracing::info;

#[derive(Debug)]
pub struct RateLimiter {
  pub qps: u64,
  pub counter: Mutex<HashMap<String, (usize, Instant)>>,
}

impl RateLimiter {
  fn new(qps: u64) -> Self {
    RateLimiter {
      qps,
      counter: Mutex::new(HashMap::new()),
    }
  }
  pub fn check_and_update(&self, client_ip: &str, count: usize) -> bool {
    let mut counter = self.counter.lock().unwrap();
    counter.retain(|_, &mut (_, timestamp)| timestamp.elapsed() < Duration::from_secs(self.qps));
    match counter.get_mut(client_ip) {
      Some((cnt, timestamp)) => {
        if *cnt >= count {
          false
        } else {
          *cnt += 1;
          *timestamp = Instant::now();
          true
        }
      }
      None => {
        counter.insert(client_ip.to_string(), (1, Instant::now()));
        true
      }
    }
  }
}

#[derive(Clone)]
pub struct CommentCache {
  pub cache: Arc<Mutex<HashMap<(String, i32), Value>>>,
}

impl CommentCache {
  fn new() -> Self {
    CommentCache {
      cache: Arc::new(Mutex::new(HashMap::new())),
    }
  }

  pub fn get(&self, path: String, page: i32) -> Option<Value> {
    let cache = self.cache.lock().unwrap().get(&(path, page)).cloned();
    cache
  }

  pub fn insert(&mut self, path: String, page: i32, data: Value) {
    self.cache.lock().unwrap().insert((path, page), data);
  }

  pub fn invalidate(&self, path: &str) {
    let mut cache = self.cache.lock().unwrap();
    cache.retain(|(old_path, _), _| old_path != path);
  }

  pub fn clear(&self) {
    self.cache.lock().unwrap().clear();
  }
}

#[derive(Clone)]
pub struct AppState {
  pub repo: RepositoryManager,
  pub rate_limiter: Arc<RateLimiter>,
  pub jwt_token: String,
  pub levels: Option<String>,
  pub comment_audit: bool,
  pub login: String,
  pub forbidden_words: Vec<String>,
  pub disable_useragent: bool,
  pub disable_region: bool,
  pub comment_cache: Arc<Mutex<CommentCache>>,
  pub ip2region: Option<Ip2Region>,
}

async fn health_check() -> HttpResponse {
  HttpResponse::Ok().json(serde_json::json!({"status": "OK"}))
}

pub fn config_app(cfg: &mut ServiceConfig) {
  cfg.service(
    web::scope("/api")
      .configure(article::config)
      .configure(comment::config)
      .configure(user::config)
      .configure(migration::config)
      .route("/health", web::get().to(health_check)),
  );
  cfg.route("/ui", web::get().to(ui_page));
  cfg.service(web::scope("/ui").configure(ui::config));
}

pub async fn start() -> impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static {
  let EnvConfig {
    database_url,
    jwt_token,
    levels,
    akismet_key,
    ipqps,
    comment_audit,
    login,
    forbidden_words,
    disable_useragent,
    disable_region,
    ip2region_db,
    ..
  } = EnvConfig::load_env().unwrap();
  let conn = Database::connect(database_url).await.unwrap();
  conn.ping().await.unwrap();
  let comment_cache = CommentCache::new();
  let mut ip2region = None;

  if akismet_key != "false" {
    info!("The anti-spam system has been activated")
  }

  if let Some(ip2region_db) = ip2region_db {
    ip2region = Ip2Region::new(&ip2region_db).ok();
  } else {
    tracing::info!("The ip region cannot be obtained because xdb is not provided!")
  }
  let state = AppState {
    repo: RepositoryManager::new(conn.clone()),
    jwt_token,
    levels,
    login,
    comment_audit,
    forbidden_words,
    disable_useragent,
    disable_region,
    ip2region,
    comment_cache: Arc::new(Mutex::new(comment_cache)),
    rate_limiter: Arc::new(RateLimiter::new(ipqps)),
  };

  move |cfg: &mut ServiceConfig| {
    cfg.service(
      web::scope("")
        .wrap(middleware::Logger::default())
        .wrap(Cors::permissive())
        .app_data(web::Data::new(state.clone()))
        .configure(config_app),
    );
  }
}
