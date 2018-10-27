//! # Asynchronous wrapper for DNS-SD C libraries
//!
//! Interesting entry points:
//!
//! * [Browse for available services][`browse`]
//! * [Create Connection to register records with][`connect`]
//! * [Enumerate domains that are recommended for registration or browsing][`enumerate_domains`]
//! * [Query for an arbitrary DNS record][`query_record`]
//! * [Register a service][`register`]
//! * [Add a record to a registered service][`Registration::add_record`]
//! * [Register record][`Connection::register_record`]
//! * [Find hostname and port (and more) for a service][`resolve`]
//!
//! Also the following things might be interesting:
//!
//! * [Purge record from cache][`reconfirm_record`]
//! * [Construct full name][`FullName::construct`]
//! * [Stream timeouts][`TimeoutStream`]
//!
//! ## Porting from dnssd C API
//!
//! | C API                           | functionality in this crate                                  |
//! |---------------------------------|--------------------------------------------------------------|
//! | [`DNSServiceAddRecord`]         | [`Registration::add_record`], [`Register::add_record`]       |
//! | [`DNSServiceBrowse`]            | [`browse`]                                                   |
//! | [`DNSServiceConstructFullName`] | [`FullName::construct`]                                      |
//! | [`DNSServiceCreateConnection`]  | [`connect`]                                                  |
//! | [`DNSServiceEnumerateDomains`]  | [`enumerate_domains`]                                        |
//! | [`DNSServiceQueryRecord`]       | [`query_record`]                                             |
//! | [`DNSServiceReconfirmRecord`]   | [`reconfirm_record`]                                         |
//! | [`DNSServiceRegister`]          | [`register`]                                                 |
//! | [`DNSServiceRegisterRecord`]    | [`Connection::register_record`]                              |
//! | [`DNSServiceResolve`]           | [`resolve`]                                                  |
//! | [`DNSServiceUpdateRecord`]      | [`Record::update_record`], [`RegisterRecord::update_record`] |
//!
//! The following functions are called automatically when needed:
//! * [`DNSServiceProcessResult`] driving callbacks (event loop)
//! * [`DNSServiceRefDeallocate`] called when dropping various resource handles
//! * [`DNSServiceRefSockFD`] used for integration with tokio (event loop)
//! * [`DNSServiceRemoveRecord`] called when dropping [`Record`](struct.Record.html)
//!
//! [`DNSServiceAddRecord`]: https://developer.apple.com/documentation/dnssd/1804730-dnsserviceaddrecord
//! [`DNSServiceBrowse`]: https://developer.apple.com/documentation/dnssd/1804742-dnsservicebrowse
//! [`DNSServiceConstructFullName`]: https://developer.apple.com/documentation/dnssd/1804753-dnsserviceconstructfullname
//! [`DNSServiceCreateConnection`]: https://developer.apple.com/documentation/dnssd/1804724-dnsservicecreateconnection
//! [`DNSServiceEnumerateDomains`]: https://developer.apple.com/documentation/dnssd/1804754-dnsserviceenumeratedomains
//! [`DNSServiceQueryRecord`]: https://developer.apple.com/documentation/dnssd/1804747-dnsservicequeryrecordc
//! [`DNSServiceReconfirmRecord`]: https://developer.apple.com/documentation/dnssd/1804726-dnsservicereconfirmrecord
//! [`DNSServiceRegister`]: https://developer.apple.com/documentation/dnssd/1804733-dnsserviceregister
//! [`DNSServiceRegisterRecord`]: https://developer.apple.com/documentation/dnssd/1804727-dnsserviceregisterrecord
//! [`DNSServiceResolve`]: https://developer.apple.com/documentation/dnssd/1804744-dnsserviceresolve
//! [`DNSServiceUpdateRecord`]: https://developer.apple.com/documentation/dnssd/1804739-dnsserviceupdaterecord
//! [`DNSServiceProcessResult`]: https://developer.apple.com/documentation/dnssd/1804696-dnsserviceprocessresult
//! [`DNSServiceRefDeallocate`]: https://developer.apple.com/documentation/dnssd/1804697-dnsservicerefdeallocate
//! [`DNSServiceRefSockFD`]: https://developer.apple.com/documentation/dnssd/1804698-dnsservicerefsockfd
//! [`DNSServiceRemoveRecord`]: https://developer.apple.com/documentation/dnssd/1804736-dnsserviceremoverecord
//! [`Registration::add_record`]: struct.Registration.html#method.add_record
//! [`Register::add_record`]: struct.Register.html#method.add_record
//! [`browse`]: fn.browse.html
//! [`FullName::construct`]: struct.FullName.html#method.construct
//! [`connect`]: fn.connect.html
//! [`enumerate_domains`]: fn.enumerate_domains.html
//! [`query_record`]: fn.query_record.html
//! [`reconfirm_record`]: fn.reconfirm_record.html
//! [`register`]: fn.register.html
//! [`Connection::register_record`]: struct.Connection.html#method.register_record
//! [`resolve`]: fn.resolve.html
//! [`Record::update_record`]: struct.Record.html#method.update_record
//! [`RegisterRecord::update_record`]: struct.RegisterRecord.html#method.update_record
//! [`TimeoutStream`]: struct.TimeoutStream.html

#![warn(missing_docs)]

extern crate futures;
#[cfg(windows)] // only the windows event loop has debug logging for now
#[macro_use]
extern crate log;
extern crate mio;
extern crate tokio_core;

#[cfg(windows)]
extern crate libc;
#[cfg(windows)]
extern crate ws2_32;
#[cfg(windows)]
extern crate winapi;

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
