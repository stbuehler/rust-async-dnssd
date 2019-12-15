pub use self::{
	browse::*,
	connection::*,
	enumerate_domains::*,
	query_record::*,
	records::Record,
	register::*,
	resolve::*,
	resolve_host::*,
};

mod browse;
mod connection;
mod enumerate_domains;
mod query_record;
mod records;
mod register;
mod resolve;
mod resolve_host;

use crate::dns_consts::{
	Class,
	Type,
};

/// Purge record from cache
///
/// See [`DNSServiceReconfirmRecord`](https://developer.apple.com/documentation/dnssd/1804726-dnsservicereconfirmrecord).
pub fn reconfirm_record(
	interface: crate::interface::Interface,
	fullname: &str,
	rr_type: Type,
	rr_class: Class,
	rdata: &[u8],
) -> ::std::io::Result<()> {
	crate::init();

	let fullname = crate::cstr::CStr::from(&fullname)?;
	crate::raw::reconfirm_record(
		0, // no flags
		interface.into_raw(),
		&fullname,
		rr_type,
		rr_class,
		rdata,
	);

	Ok(())
}

/// Full name consiting of (up to) three parts
pub struct FullName<'a> {
	/// (unescaped) service name (becomes single label in full name)
	pub service: Option<&'a str>,
	/// registration type (valid names don't need escaping)
	pub reg_type: &'a str,
	/// (escaped) domain name (most names don't need escaping)
	pub domain: &'a str,
}

impl<'a> FullName<'a> {
	/// Escape and concatenate all three parts to a full name
	///
	/// See [`DNSServiceConstructFullName`](https://developer.apple.com/documentation/dnssd/1804753-dnsserviceconstructfullname)
	pub fn construct(&self) -> ::std::io::Result<String> {
		use std::io;

		let service = crate::cstr::NullableCStr::from(&self.service)?;
		let reg_type = crate::cstr::CStr::from(&self.reg_type)?;
		let domain = crate::cstr::CStr::from(&self.domain)?;

		const SIZE: usize = crate::ffi::MAX_DOMAIN_NAME + 200;
		let mut buf: Vec<u8> = Vec::new();
		buf.reserve(SIZE);
		let len = unsafe {
			crate::ffi::DNSServiceConstructFullName(
				buf.as_mut_ptr() as *mut i8,
				service.as_ptr(),
				reg_type.as_ptr(),
				domain.as_ptr(),
			)
		};

		if len < 0 {
			return Err(io::Error::new(io::ErrorKind::InvalidInput, "invalid input"));
		}

		unsafe {
			buf.set_len(len as usize);
		}

		String::from_utf8(buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))
	}
}
