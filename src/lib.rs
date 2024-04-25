//! # masterstat
//!
//! Get server addresses from QuakeWorld master servers.

mod command;
mod server_address;
mod udp;

pub use crate::command::server_addresses;
pub use crate::command::server_addresses_from_many;
pub use crate::server_address::ServerAddress;
