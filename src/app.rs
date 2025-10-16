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
  middlewares::SecureDomians,
  migration::migrate,
  repository::RepositoryManager,
};

use actix_cors::Cors;
use actix_web::{
  App, HttpMessage, HttpRequest, HttpResponse, HttpServer,
  dev::Service,
  http::header::USER_AGENT,
  middleware,
  web::{self, ServiceConfig},
};
use serde_json::Value;
use tracing::info;

#[derive(Debug)]
pub struct RateLimiter {
  qps: u64,
  counter: Mutex<HashMap<String, (usize, Instant)>>,
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
    self.cache.lock().unwrap().get(&(path, page)).cloned()
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
  pub site_url: String,
  pub site_name: String,
}

async fn health_check(req: HttpRequest) -> HttpResponse {
  let extensions = req.extensions();
  let host = extensions.get::<String>();
  HttpResponse::Ok().json(serde_json::json!({"status": "OK", "header": host}))
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
  #[cfg(feature = "leancloud")]
  cfg.route("/", web::get().to(health_check));
}

pub async fn start() -> Result<(), AppError> {
  let EnvConfig {
    workers,
    host,
    port,
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
    secure_domains,
    site_name,
    site_url,
    ..
  } = EnvConfig::load_env()?;
  let conn = migrate(&database_url).await?;
  conn.ping().await?;
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
    site_url,
    site_name,
    comment_cache: Arc::new(Mutex::new(comment_cache)),
    rate_limiter: Arc::new(RateLimiter::new(ipqps)),
  };
  Ok(
    HttpServer::new(move || {
      App::new()
        .wrap_fn(|req, srv| {
          if let Some(host_header) = req.headers().get(USER_AGENT) {
            if let Ok(host_value) = host_header.to_str() {
              req.extensions_mut().insert(host_value.to_string());
            }
          }
          let fut = srv.call(req);
          async { fut.await }
        })
        .wrap(SecureDomians::new(secure_domains.clone()))
        .wrap(middleware::Logger::default())
        .wrap(Cors::permissive())
        .app_data(web::Data::new(state.clone()))
        .configure(config_app)
    })
    .bind((host, port))?
    .workers(workers)
    .run()
    .await?,
  )
}
