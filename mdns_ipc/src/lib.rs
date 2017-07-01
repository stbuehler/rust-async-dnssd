extern crate mdns_ipc_core;
#[macro_use]
extern crate mdns_ipc_derive;
#[macro_use]
extern crate log;
extern crate futures;
extern crate tokio_core;
extern crate tokio_io;

pub use mdns_ipc_core::bytes;

pub use self::connection::*;
pub use self::enums::*;
pub use self::rrdata::RRData;
pub use self::structs::*;

pub mod errors;

mod connection;
mod enums;
mod rrdata;
mod reader;
mod status;
mod structs;
