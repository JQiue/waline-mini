use crate::entities::wl_users;
use sea_orm::{
  ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, QueryOrder,
};

pub enum UserQueryBy<'a> {
  Id(u32),
  Email(&'a str),
}

pub enum UserQueryBySocial {
  QQ,
  Weibo,
  Github,
  Twitter,
  Facebook,
}

#[derive(Debug, Clone)]
pub struct UserRepository<'a> {
  pub db: &'a DatabaseConnection,
}

impl<'a> UserRepository<'a> {
  pub async fn _get_users(&self) -> Result<Vec<wl_users::Model>, DbErr> {
    wl_users::Entity::find().all(self.db).await
  }

  pub async fn get_user_by_email(&self, email: &str) -> Result<Option<wl_users::Model>, DbErr> {
    self.get_user(UserQueryBy::Email(email)).await
  }

  pub async fn get_user_by_id(&self, id: u32) -> Result<Option<wl_users::Model>, DbErr> {
    self.get_user(UserQueryBy::Id(id)).await
  }

  pub async fn get_user_by_social(
    &self,
    r#type: UserQueryBySocial,
    id: &str,
  ) -> Result<Option<wl_users::Model>, DbErr> {
    let mut select = wl_users::Entity::find();
    match r#type {
      UserQueryBySocial::QQ => select = select.filter(wl_users::Column::Qq.eq(id)),
      UserQueryBySocial::Weibo => select = select.filter(wl_users::Column::Weibo.eq(id)),
      UserQueryBySocial::Github => select = select.filter(wl_users::Column::Github.eq(id)),
      UserQueryBySocial::Twitter => select = select.filter(wl_users::Column::Twitter.eq(id)),
      UserQueryBySocial::Facebook => select = select.filter(wl_users::Column::Facebook.eq(id)),
    }
    select.one(self.db).await
  }

  async fn get_user(&self, query_by: UserQueryBy<'a>) -> Result<Option<wl_users::Model>, DbErr> {
    let mut select = wl_users::Entity::find();
    match query_by {
      UserQueryBy::Id(id) => select = select.filter(wl_users::Column::Id.eq(id)),
      UserQueryBy::Email(email) => select = select.filter(wl_users::Column::Email.eq(email)),
    }
    select.one(self.db).await
  }

  pub async fn is_admin_user(&self, email: &str) -> Result<bool, DbErr> {
    let user = wl_users::Entity::find()
      .filter(wl_users::Column::Email.eq(email))
      .one(self.db)
      .await?;
    match user {
      Some(user) => Ok(user.user_type == "administrator"),
      None => Ok(false),
    }
  }

  pub async fn create_user(&self, user: wl_users::ActiveModel) -> Result<wl_users::Model, DbErr> {
    user.insert(self.db).await
  }

  pub async fn update_user(&self, user: wl_users::ActiveModel) -> Result<wl_users::Model, DbErr> {
    user.update(self.db).await
  }

  pub async fn set_2fa(&self, user: wl_users::ActiveModel) -> Result<wl_users::Model, DbErr> {
    user.update(self.db).await
  }

  pub async fn _has_user_by_id(&self, id: u32) -> Result<bool, DbErr> {
    self.has_user(UserQueryBy::Id(id)).await
  }

  pub async fn has_user_by_email(&self, email: &str) -> Result<bool, DbErr> {
    self.has_user(UserQueryBy::Email(email)).await
  }

  async fn has_user(&self, query_by: UserQueryBy<'a>) -> Result<bool, DbErr> {
    let mut select = wl_users::Entity::find();
    match query_by {
      UserQueryBy::Id(id) => select = select.filter(wl_users::Column::Id.eq(id)),
      UserQueryBy::Email(email) => select = select.filter(wl_users::Column::Email.eq(email)),
    }
    let res = select.one(self.db).await?;
    Ok(res.is_some())
  }

  pub async fn is_first_user(&self) -> Result<bool, DbErr> {
    let users = wl_users::Entity::find().all(self.db).await?;
    Ok(users.is_empty())
  }

  pub async fn is_first_admin_user(&self, id: u32) -> Result<bool, DbErr> {
    let users = wl_users::Entity::find()
      .filter(wl_users::Column::UserType.eq("administrator"))
      .order_by_asc(wl_users::Column::CreatedAt)
      .all(self.db)
      .await?;
    if let Some(first_user) = users.first() {
      Ok(first_user.id == id)
    } else {
      Ok(false)
    }
  }
}
