use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(WlUsers::Table)
          .if_not_exists()
          .col(pk_auto(WlUsers::Id).unsigned())
          .col(string(WlUsers::DisplayName))
          .col(string(WlUsers::Email))
          .col(string(WlUsers::Password))
          .col(string(WlUsers::Type))
          .col(string_null(WlUsers::Label))
          .col(string_null(WlUsers::Url))
          .col(string_null(WlUsers::Avatar))
          .col(string_null(WlUsers::Github))
          .col(string_null(WlUsers::Twitter))
          .col(string_null(WlUsers::Facebook))
          .col(string_null(WlUsers::Google))
          .col(string_null(WlUsers::Weibo))
          .col(string_null(WlUsers::Qq))
          .col(string_null(WlUsers::TwoFactorAuth))
          .col(timestamp(WlUsers::CreatedAt).default(Expr::current_timestamp()))
          .col(timestamp_null(WlUsers::UpdatedAt).default(Expr::current_timestamp()))
          .to_owned(),
      )
      .await?;
    manager
      .create_table(
        Table::create()
          .table(WlComment::Table)
          .if_not_exists()
          .col(pk_auto(WlComment::Id).unsigned())
          .col(integer_null(WlComment::UserId))
          .col(string_null(WlComment::Comment))
          .col(string_null(WlComment::Ip))
          .col(string_null(WlComment::Link))
          .col(string_null(WlComment::Mail))
          .col(string_null(WlComment::Nick))
          .col(integer_null(WlComment::Pid))
          .col(integer_null(WlComment::Rid))
          .col(tiny_integer_null(WlComment::Sticky))
          .col(string_null(WlComment::Status))
          .col(integer_null(WlComment::Like))
          .col(string_null(WlComment::Ua))
          .col(string_null(WlComment::Url))
          .col(timestamp_null(WlComment::CreatedAt).default(Expr::current_timestamp()))
          .col(timestamp_null(WlComment::UpdatedAt).default(Expr::current_timestamp()))
          .col(timestamp_null(WlComment::InsertedAt).default(Expr::current_timestamp()))
          .to_owned(),
      )
      .await?;
    manager
      .create_table(
        Table::create()
          .table(WlCounter::Table)
          .if_not_exists()
          .col(pk_auto(WlCounter::Id).unsigned())
          .col(string(WlCounter::Url))
          .col(integer_null(WlCounter::Time))
          .col(integer_null(WlCounter::Reaction0))
          .col(integer_null(WlCounter::Reaction1))
          .col(integer_null(WlCounter::Reaction2))
          .col(integer_null(WlCounter::Reaction3))
          .col(integer_null(WlCounter::Reaction4))
          .col(integer_null(WlCounter::Reaction5))
          .col(integer_null(WlCounter::Reaction6))
          .col(integer_null(WlCounter::Reaction7))
          .col(integer_null(WlCounter::Reaction8))
          .col(timestamp_null(WlCounter::CreatedAt).default(Expr::current_timestamp()))
          .col(timestamp_null(WlCounter::UpdatedAt).default(Expr::current_timestamp()))
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_table(Table::drop().table(WlUsers::Table).to_owned())
      .await?;
    manager
      .drop_table(Table::drop().table(WlComment::Table).to_owned())
      .await?;
    manager
      .drop_table(Table::drop().table(WlCounter::Table).to_owned())
      .await
  }
}

#[derive(DeriveIden)]
enum WlUsers {
  Table,
  Id,
  DisplayName,
  Email,
  Password,
  Type,
  Label,
  Url,
  Avatar,
  Github,
  Twitter,
  Facebook,
  Google,
  Weibo,
  Qq,
  #[sea_orm(iden = "2fa")]
  TwoFactorAuth,
  #[sea_orm(iden = "createdAt")]
  CreatedAt,
  #[sea_orm(iden = "updatedAt")]
  UpdatedAt,
}

#[derive(DeriveIden)]
enum WlComment {
  Table,
  Id,
  UserId,
  Comment,
  Ip,
  Link,
  Mail,
  Nick,
  Pid,
  Rid,
  Sticky,
  Status,
  Like,
  Ua,
  Url,
  #[sea_orm(iden = "insertedAt")]
  InsertedAt,
  #[sea_orm(iden = "createdAt")]
  CreatedAt,
  #[sea_orm(iden = "updatedAt")]
  UpdatedAt,
}

#[derive(DeriveIden)]
enum WlCounter {
  Table,
  Id,
  Time,
  Url,
  Reaction0,
  Reaction1,
  Reaction2,
  Reaction3,
  Reaction4,
  Reaction5,
  Reaction6,
  Reaction7,
  Reaction8,
  #[sea_orm(iden = "createdAt")]
  CreatedAt,
  #[sea_orm(iden = "updatedAt")]
  UpdatedAt,
}
