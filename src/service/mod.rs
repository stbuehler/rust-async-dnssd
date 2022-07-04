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
use std::ffi::{
	CStr,
	CString,
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
use std::os::raw::c_char;

/// Purge record from cache
///
/// See [`DNSServiceReconfirmRecord`](https://developer.apple.com/documentation/dnssd/1804726-dnsservicereconfirmrecord).
#[doc(alias = "DNSServiceReconfirmRecord")]
pub fn reconfirm_record(
	interface: crate::interface::Interface,
	fullname: &str,
	rr_type: Type,
	rr_class: Class,
	rdata: &[u8],
) -> ::std::io::Result<()> {
	crate::init();

	let fullname = crate::cstr::CStr::from(&fullname)?;
	crate::inner::reconfirm_record(
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
	#[doc(alias = "DNSServiceConstructFullName")]
	pub fn construct(&self) -> ::std::io::Result<String> {
		use std::io;

		let service = crate::cstr::NullableCStr::from(&self.service)?;
		let reg_type = crate::cstr::CStr::from(&self.reg_type)?;
		let domain = crate::cstr::CStr::from(&self.domain)?;

		const SIZE: usize = crate::ffi::MAX_DOMAIN_NAME;
		let mut buf: Vec<u8> = Vec::new();
		buf.resize(SIZE, 0);
		let result = unsafe {
			crate::ffi::DNSServiceConstructFullName(
				buf.as_mut_ptr() as *mut c_char,
				service.as_ptr(),
				reg_type.as_ptr(),
				domain.as_ptr(),
			)
		};

		if result != 0 {
			return Err(io::Error::new(io::ErrorKind::InvalidInput, "invalid input"));
		}

		let nul_pos = buf.iter().position(|&x| x == 0);
		match nul_pos {
			Some(nul_pos) => {
				let subslice = &buf[..nul_pos + 1];
				// SAFETY: We know there is a nul byte at nul_pos, so this slice
				// (ending at the nul byte) is a well-formed C string.
				unsafe {
					CString::from(CStr::from_bytes_with_nul_unchecked(subslice))
						.into_string()
						.map_err(|e| io::Error::new(io::ErrorKind::Other, e))
				}
			},
			None => Err(io::Error::new(io::ErrorKind::InvalidInput, "invalid input")),
		}
	}
}
