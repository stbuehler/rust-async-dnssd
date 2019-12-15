use futures::{
	self,
	Async,
};
use std::{
	io,
	os::raw::{
		c_char,
		c_void,
	},
};

use crate::{
	cstr,
	ffi,
	interface::Interface,
	raw,
	service::{
		resolve_host_extended,
		ResolveHost,
		ResolveHostData,
	},
};

type CallbackStream = crate::stream::ServiceStream<ResolveResult>;

/// Pending resolve request
#[must_use = "streams do nothing unless polled"]
pub struct Resolve(CallbackStream);

impl futures::Stream for Resolve {
	type Error = io::Error;
	type Item = ResolveResult;

	fn poll(&mut self) -> Result<Async<Option<Self::Item>>, Self::Error> {
		self.0.poll()
	}
}

/// Resolve result
///
/// See [`DNSServiceResolveReply`](https://developer.apple.com/documentation/dnssd/dnsserviceresolvereply).
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct ResolveResult {
	/// interface service was resolved on
	pub interface: Interface,
	/// full name of service
	pub fullname: String,
	/// hostname the service is provided on
	pub host_target: String,
	/// port the service is provided on (native endian)
	pub port: u16,
	/// TXT RDATA describing service parameters
	pub txt: Vec<u8>,
}

impl ResolveResult {
	/// Lookup socket addresses for resolved service
	pub fn resolve_socket_address(&self) -> io::Result<ResolveHost> {
		let rhdata = ResolveHostData {
			interface: self.interface,
			..Default::default()
		};
		resolve_host_extended(&self.host_target, self.port, rhdata)
	}
}

extern "C" fn resolve_callback(
	_sd_ref: ffi::DNSServiceRef,
	_flags: ffi::DNSServiceFlags,
	interface_index: u32,
	error_code: ffi::DNSServiceErrorType,
	fullname: *const c_char,
	host_target: *const c_char,
	port: u16,
	txt_len: u16,
	txt_record: *const u8,
	context: *mut c_void,
) {
	CallbackStream::run_callback(context, error_code, || {
		let fullname = unsafe { cstr::from_cstr(fullname) }?;
		let host_target = unsafe { cstr::from_cstr(host_target) }?;
		let txt = unsafe { ::std::slice::from_raw_parts(txt_record, txt_len as usize) };

		Ok(ResolveResult {
			interface: Interface::from_raw(interface_index),
			fullname: fullname.to_string(),
			host_target: host_target.to_string(),
			port: u16::from_be(port),
			txt: txt.into(),
		})
	});
}

/// Find hostname and port (and more) for a service
///
/// You probably want to use [`BrowseResult::resolve`] instead.
///
/// See [`DNSServiceResolve`](https://developer.apple.com/documentation/dnssd/1804744-dnsserviceresolve).
///
/// [`BrowseResult::resolve`]: struct.BrowseResult.html#method.resolve
pub fn resolve(
	interface: Interface,
	name: &str,
	reg_type: &str,
	domain: &str,
) -> io::Result<Resolve> {
	crate::init();

	let name = cstr::CStr::from(&name)?;
	let reg_type = cstr::CStr::from(&reg_type)?;
	let domain = cstr::CStr::from(&domain)?;

	Ok(Resolve(CallbackStream::new(move |sender| {
		raw::DNSService::resolve(
			0, // no flags
			interface.into_raw(),
			&name,
			&reg_type,
			&domain,
			Some(resolve_callback),
			sender,
		)
	})?))
}
