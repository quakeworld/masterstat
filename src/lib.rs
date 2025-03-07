//! # masterstat
//!
//! Get server addresses from QuakeWorld master servers.

mod command;
mod server_address;
mod tinyudp;

pub use crate::command::{server_addresses, server_addresses_from_many};
pub use crate::server_address::ServerAddress;
