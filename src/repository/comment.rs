use crate::entities::wl_comment;
use sea_orm::{
  ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, DeleteResult, EntityTrait,
  ItemsAndPagesNumber, Order, PaginatorTrait, QueryFilter, QueryOrder,
};

#[derive(Debug, Clone)]
pub struct CommentRepository<'a> {
  pub db: &'a DatabaseConnection,
}

impl CommentRepository<'_> {
  pub async fn get_comment(&self, id: u32) -> Result<Option<wl_comment::Model>, DbErr> {
    wl_comment::Entity::find()
      .filter(wl_comment::Column::Id.eq(id))
      .one(self.db)
      .await
  }

  pub async fn get_comments_for_user(
    &self,
    path: &str,
    sort_col: wl_comment::Column,
    sort_ord: Order,
    page: u64,
    page_size: u64,
  ) -> Result<(ItemsAndPagesNumber, Vec<wl_comment::Model>), DbErr> {
    let paginator = wl_comment::Entity::find()
      .filter(wl_comment::Column::Url.contains(path))
      .filter(wl_comment::Column::Pid.is_null())
      .filter(wl_comment::Column::Status.is_not_in(["waiting", "spam"]))
      .order_by(sort_col, sort_ord)
      .paginate(self.db, page_size);
    let parrent_comments = paginator.fetch_page(page - 1).await?;
    Ok((paginator.num_items_and_pages().await?, parrent_comments))
  }

  async fn get_comments_for_admin(
    &self,
    path: &str,
    sort_col: wl_comment::Column,
    sort_ord: Order,
    page: u64,
    page_size: u64,
  ) -> Result<(ItemsAndPagesNumber, Vec<wl_comment::Model>), DbErr> {
    let paginator = wl_comment::Entity::find()
      .filter(wl_comment::Column::Url.contains(path))
      .filter(wl_comment::Column::Pid.is_null())
      .order_by(sort_col, sort_ord)
      .paginate(self.db, page_size);
    let parrent_comments = paginator.fetch_page(page - 1).await?;
    Ok((paginator.num_items_and_pages().await?, parrent_comments))
  }

  pub async fn get_root_comment(
    &self,
    path: &str,
    sort_by: String,
    page: u64,
    page_size: u64,
    is_admin: bool,
  ) -> Result<(ItemsAndPagesNumber, Vec<wl_comment::Model>), DbErr> {
    let (sort_col, sort_ord) = match sort_by.as_str() {
      "insertedAt_asc" => (wl_comment::Column::InsertedAt, Order::Asc),
      "like_desc" => (wl_comment::Column::Like, Order::Desc),
      _ => (wl_comment::Column::InsertedAt, Order::Desc),
    };
    if is_admin {
      self
        .get_comments_for_admin(path, sort_col, sort_ord, page, page_size)
        .await
    } else {
      self
        .get_comments_for_user(path, sort_col, sort_ord, page, page_size)
        .await
    }
  }

  pub async fn create_comment(
    &self,
    comment: wl_comment::ActiveModel,
  ) -> Result<wl_comment::Model, DbErr> {
    comment.insert(self.db).await
  }

  pub async fn update_comment(
    &self,
    comment: wl_comment::ActiveModel,
  ) -> Result<wl_comment::Model, DbErr> {
    comment.update(self.db).await
  }

  pub async fn delete_comment(&self, id: u32) -> Result<DeleteResult, DbErr> {
    wl_comment::Entity::delete_by_id(id).exec(self.db).await
  }

  pub async fn is_comment_owner(&self, id: u32, user_id: u32) -> Result<bool, DbErr> {
    Ok(
      wl_comment::Entity::find()
        .filter(wl_comment::Column::Id.eq(id))
        .filter(wl_comment::Column::UserId.eq(user_id))
        .one(self.db)
        .await?
        .is_some(),
    )
  }

  pub async fn is_duplicate(
    &self,
    url: &str,
    mail: &str,
    nick: &str,
    link: Option<&str>,
    comment: &str,
  ) -> Result<bool, DbErr> {
    let res = wl_comment::Entity::find()
      .filter(wl_comment::Column::Url.eq(url))
      .filter(wl_comment::Column::Mail.eq(mail))
      .filter(wl_comment::Column::Nick.eq(nick))
      .filter(wl_comment::Column::Link.eq(link))
      .filter(wl_comment::Column::Comment.eq(comment))
      .all(self.db)
      .await?;
    Ok(!res.is_empty())
  }

  pub async fn is_anonymous(&self, comment_id: u32) -> Result<bool, DbErr> {
    let res = wl_comment::Entity::find_by_id(comment_id)
      .filter(wl_comment::Column::UserId.is_not_null().ne(""))
      .one(self.db)
      .await?;
    Ok(res.is_none())
  }

  pub async fn get_comment_count_by_nick_and_mail(
    &self,
    nick: Option<String>,
    mail: Option<String>,
  ) -> Result<u64, DbErr> {
    wl_comment::Entity::find()
      .filter(wl_comment::Column::Nick.eq(nick))
      .filter(wl_comment::Column::Mail.eq(mail))
      .count(self.db)
      .await
  }

  pub async fn get_subcomments(
    &self,
    path: &str,
    pid: u32,
    is_admin: bool,
  ) -> Result<Vec<wl_comment::Model>, DbErr> {
    let mut select = wl_comment::Entity::find()
      .filter(wl_comment::Column::Url.contains(path))
      .filter(wl_comment::Column::Pid.eq(pid))
      .order_by(wl_comment::Column::InsertedAt, Order::Asc);

    if !is_admin {
      select = select.filter(wl_comment::Column::Status.is_not_in(["waiting", "spam"]));
    }

    select.all(self.db).await
  }

  pub async fn get_comments_list_by_admin(
    &self,
    email: &str,
    status: &str,
    keyword: &str,
    page: u64,
    page_size: u64,
    owner: String,
  ) -> Result<(ItemsAndPagesNumber, Vec<wl_comment::Model>), DbErr> {
    let mut select = wl_comment::Entity::find()
      .filter(wl_comment::Column::Status.eq(status))
      .filter(wl_comment::Column::Comment.contains(keyword));
    if owner == "mine" {
      select = select.filter(wl_comment::Column::Mail.eq(email))
    }
    let paginator = select.paginate(self.db, page_size);
    let comment = paginator.fetch_page(page - 1).await?;
    Ok((paginator.num_items_and_pages().await?, comment))
  }
}
