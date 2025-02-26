use serde::Deserialize;

#[derive(Deserialize)]
pub struct UserRegisterQuery {
  pub lang: String,
}

#[derive(Deserialize)]
pub struct UserRegisterBody {
  pub display_name: String,
  pub email: String,
  pub password: String,
  pub url: String,
}

#[derive(Deserialize)]
pub struct UserLoginBody {
  pub code: String,
  pub email: String,
  pub password: String,
}

#[derive(Deserialize)]
pub struct SetUserProfileBody {
  pub display_name: Option<String>,
  pub label: Option<String>,
  pub url: Option<String>,
  pub password: Option<String>,
  pub avatar: Option<String>,
  #[serde(rename = "2fa")]
  pub two_factor_auth: Option<String>,
}

#[derive(Deserialize)]
pub struct GetUserQuery {
  pub email: Option<String>,
  pub page: Option<u32>,
  pub lang: String,
}

#[derive(Deserialize)]
pub struct VerificationQuery {
  pub token: String,
  pub email: String,
}

#[derive(Deserialize)]
pub struct Set2faBody {
  pub code: String,
  pub secret: String,
}

#[derive(Deserialize)]
pub struct Get2faQuery {
  pub lang: String,
  pub email: Option<String>,
}

#[derive(Deserialize)]
pub struct SetUserTypeBody {
  pub r#type: String,
}

#[derive(Deserialize)]
pub struct UserPasswordQuery {
  pub lang: String,
}

#[derive(Deserialize)]
pub struct UserPasswordBody {
  pub email: String,
}
