//! # masterstat
//!
//! Get server addresses from QuakeWorld master servers.

mod query;
mod query_multiple;
mod server_address;

pub use crate::query::query;
pub use crate::query_multiple::{MultiQueryResult, query_multiple};
