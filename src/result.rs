use crate::error::DynError;

pub type AppResult<T> = Result<T, Box<DynError>>;
