pub mod ast;
mod token;

#[salsa::query_group(FrontendDatabaseStorage)]
pub trait FrontendDatabase: salsa::Database {}
