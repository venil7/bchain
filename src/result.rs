use crate::error::AppError;
use std::error::Error;

pub type AppResult<T> = Result<T, AppError>;
pub type DynResult<T> = Result<T, Box<dyn Error>>;
