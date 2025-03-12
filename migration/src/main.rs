pub use sea_orm_migration::prelude::*;

mod migration_01_init_table;

#[async_std::main]
async fn main() {
  cli::run_cli(migration::Migrator).await;
}
