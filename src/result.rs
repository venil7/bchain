use crate::error::AppError;
use std::error::Error;

pub type DynError = dyn Error + Send + Sync + 'static;
pub type AppResult<T> = Result<T, Box<DynError>>;
