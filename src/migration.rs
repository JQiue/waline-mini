use migration::{Migrator, MigratorTrait};
use sea_orm::{DatabaseConnection, DbErr};

pub async fn migrate(database_url: &str) -> Result<DatabaseConnection, DbErr> {
  let connection = sea_orm::Database::connect(database_url).await?;
  Migrator::up(&connection, None).await?;
  Ok(connection)
}
