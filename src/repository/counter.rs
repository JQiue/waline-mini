use crate::entities::wl_counter;
use helpers::time::utc_now;
use sea_orm::{
  ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, Set,
};

#[derive(Debug, Clone)]
pub struct CounterRepository<'a> {
  pub db: &'a DatabaseConnection,
}

impl CounterRepository<'_> {
  pub async fn get_counter(&self, url: &str) -> Result<Option<wl_counter::Model>, DbErr> {
    wl_counter::Entity::find()
      .filter(wl_counter::Column::Url.eq(url))
      .one(self.db)
      .await
  }

  pub async fn create_counter(&self, url: String) -> Result<wl_counter::Model, DbErr> {
    wl_counter::ActiveModel {
      time: Set(Some(1)),
      url: Set(url),
      created_at: Set(Some(utc_now())),
      ..Default::default()
    }
    .insert(self.db)
    .await
  }

  pub async fn update_counter(
    &self,
    active_counter: wl_counter::ActiveModel,
  ) -> Result<wl_counter::Model, DbErr> {
    active_counter.update(self.db).await
  }
}
