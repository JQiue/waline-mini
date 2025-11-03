use migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectionTrait, DatabaseConnection, DbErr};
use tracing::info;

pub async fn migrate(database_url: &str) -> Result<DatabaseConnection, DbErr> {
  let connection = sea_orm::Database::connect(database_url).await?;
  match connection.get_database_backend() {
    sea_orm::DatabaseBackend::Sqlite => info!("Using SQLite"),
    sea_orm::DatabaseBackend::MySql => info!("Using MySQL/MariaDB"),
    sea_orm::DatabaseBackend::Postgres => info!("Using PostgreSQL"),
  }
  Migrator::up(&connection, None).await?;
  Ok(connection)
}
