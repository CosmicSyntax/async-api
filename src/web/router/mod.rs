// Re-export Status
mod status;
pub use crate::web::router::status::*;

mod auth;
pub use crate::web::router::auth::*;

mod register;
pub use crate::web::router::register::*;

mod stream;
pub use crate::web::router::stream::*;
