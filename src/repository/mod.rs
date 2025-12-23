mod comment;
mod counter;
pub mod user;

use sea_orm::DatabaseConnection;

pub use comment::CommentRepository;
pub use counter::CounterRepository;
pub use user::UserRepository;

#[derive(Debug, Clone)]
pub struct RepositoryManager {
  pub db: DatabaseConnection,
}

impl RepositoryManager {
  pub fn new(db: DatabaseConnection) -> Self {
    Self { db }
  }

  pub fn user(&self) -> UserRepository<'_> {
    UserRepository { db: &self.db }
  }

  pub fn comment(&self) -> CommentRepository<'_> {
    CommentRepository { db: &self.db }
  }

  pub fn counter(&self) -> CounterRepository<'_> {
    CounterRepository { db: &self.db }
  }
}
