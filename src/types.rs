use crate::error::AppError;

pub type ServiceResult<T> = Result<T, AppError>;
