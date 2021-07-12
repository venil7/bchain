// use std::error::Error;

// #[derive(Debug)]
// pub struct AppError(String);

// impl std::fmt::Display for AppError {
//   fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
//     write!(f, "app-error")
//   }
// }
// impl AppError {
//   pub fn from_string(st: &str) -> AppError {
//     AppError(st.to_owned())
//   }
// }
// impl Error for AppError {}
// impl From<pkcs8::Error> for AppError {
//   fn from(err: pkcs8::Error) -> Self {
//     AppError(format!("{}", err))
//   }
// }

// impl From<Box<dyn Error>> for AppError {
//   fn from(err: Box<dyn Error>) -> Self {
//     AppError(format!("{}", err))
//   }
// }

// impl std::convert::From<std::io::Error> for AppError {
//   fn from(err: std::io::Error) -> Self {
//     AppError(format!("{}", err))
//   }
// }
