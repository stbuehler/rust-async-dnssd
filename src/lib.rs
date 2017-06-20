//! # Asynchronous wrapper for DNS-SD C libraries
//!
//! Interesting entry points:
//!
//! * [Browses for available services](method.browse.html)
//! * [Create Connection to register records with](method.connect.html)
//! * [Enumerates domains that are recommended for registration or browsing](method.enumerate_domains.html)
//! * [Query for an arbitrary DNS record](method.query_record.html)
//! * [Registers a service](method.register.html)
//! * [Find hostname and port (and more) for a service](method.resolve.html)
//!
//! Also the following things might be interesting:
//!
//! * [Purge record from cache](method.reconfirm_record.html)
//! * [Construct full name](struct.FullName#method.construct)
//! * [Stream timeouts](struct.TimeoutStream)

#![warn(missing_docs)]

extern crate futures;
#[cfg(windows)] // only the windows event loop has debug logging for now
#[macro_use]
extern crate log;
extern crate mio;
extern crate tokio_core;

#[cfg(windows)]
extern crate libc;

pub use self::error::*;
pub use self::ffi::MAX_DOMAIN_NAME;
pub use self::interface::*;
pub use self::remote::*;
pub use self::service::*;
pub use self::timeout_stream::*;

mod flags_macro;

mod cstr;
mod error;
mod evented;
mod ffi;
mod future;
mod interface;
mod raw;
mod raw_box;
mod remote;
mod service;
mod stream;
mod timeout_stream;
