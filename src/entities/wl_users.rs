//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.11

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "wl_Users")]
pub struct Model {
  #[sea_orm(primary_key)]
  pub id: u32,
  pub display_name: String,
  pub email: String,
  pub password: String,
  pub r#type: String,
  pub label: Option<String>,
  pub url: Option<String>,
  pub avatar: Option<String>,
  pub github: Option<String>,
  pub twitter: Option<String>,
  pub facebook: Option<String>,
  pub google: Option<String>,
  pub weibo: Option<String>,
  pub qq: Option<String>,
  #[sea_orm(column_name = "2fa")]
  pub two_factor_auth: Option<String>,
  #[sea_orm(column_name = "createdAt")]
  pub created_at: Option<DateTimeUtc>,
  #[sea_orm(column_name = "updatedAt")]
  pub updated_at: Option<DateTimeUtc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
