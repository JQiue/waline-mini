use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct GetArticleQuery {
  pub path: String,
  pub r#type: String,
  pub lang: String,
}

#[derive(Deserialize)]
pub struct UpdateArticleBody {
  pub action: Option<String>,
  pub path: String,
  pub r#type: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateArticleQuery {
  pub lang: String,
}
