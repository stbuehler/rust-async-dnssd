use futures::{self,Async};
use std::os::raw::{c_void,c_char};
use std::io;
use tokio_core::reactor::{Handle,Remote};

use cstr;
use ffi;
use interface::Interface;
use raw;
use remote::GetRemote;

type CallbackStream = ::stream::ServiceStream<ResolveResult>;

/// Pending resolve request
pub struct Resolve(CallbackStream);

impl futures::Stream for Resolve {
	type Item = ResolveResult;
	type Error = io::Error;

	fn poll(&mut self) -> Result<Async<Option<Self::Item>>, Self::Error> {
		self.0.poll()
	}
}

impl GetRemote for Resolve {
	fn remote(&self) -> &Remote {
		self.0.remote()
	}
}

/// Resolve result
///
/// See [`DNSServiceResolveReply`](https://developer.apple.com/documentation/dnssd/dnsserviceresolvereply).
#[derive(Clone,PartialEq,Eq,PartialOrd,Ord,Hash,Debug)]
pub struct ResolveResult{
	///
	pub interface: Interface,
	///
	pub fullname: String,
	///
	pub host_target: String,
	///
	pub port: u16,
	///
	pub txt: Vec<u8>,
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
	context: *mut c_void
) {
	CallbackStream::run_callback(context, error_code, || {
		let fullname = unsafe { cstr::from_cstr(fullname) }?;
		let host_target = unsafe { cstr::from_cstr(host_target) }?;
		let txt = unsafe { ::std::slice::from_raw_parts(txt_record, txt_len as usize) };

		Ok(ResolveResult{
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
/// See [`DNSServiceResolve`](https://developer.apple.com/documentation/dnssd/1804744-dnsserviceresolve).
pub fn resolve(
	interface: Interface,
	name: &str,
	reg_type: &str,
	domain: &str,
	handle: &Handle
) -> io::Result<Resolve> {
	let name = cstr::CStr::from(&name)?;
	let reg_type = cstr::CStr::from(&reg_type)?;
	let domain = cstr::CStr::from(&domain)?;

	Ok(Resolve(CallbackStream::new(handle, move |sender|
		raw::DNSService::resolve(
			0, /* no flags */
			interface.into_raw(),
			&name,
			&reg_type,
			&domain,
			Some(resolve_callback),
			sender,
		)
	)?))
}
