use futures::sync::mpsc;
use futures::{self,Async};
use std::os::raw::{c_void,c_char};
use std::io;
use tokio_core::reactor::{Handle,Remote};

use cstr;
use error::Error;
use evented::EventedDNSService;
use ffi;
use interface_index::InterfaceIndex;
use raw;
use remote::GetRemote;
use stream::ServiceStream;

pub struct Resolve(ServiceStream<ResolveData>);

impl futures::Stream for Resolve {
	type Item = ResolveData;
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

#[derive(Clone,PartialEq,Eq,PartialOrd,Ord,Hash,Debug)]
pub struct ResolveData{
	pub interface_index: InterfaceIndex,
	pub fullname: String,
	pub host_target: String,
	pub port: u16,
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
	let sender = context as *mut mpsc::UnboundedSender<io::Result<ResolveData>>;
	let sender : &mpsc::UnboundedSender<io::Result<ResolveData>> = unsafe { &*sender };

	let data = Error::from(error_code).map_err(io::Error::from).and_then(|_| {
		let fullname = unsafe { cstr::from_cstr(fullname) }?;
		let host_target = unsafe { cstr::from_cstr(host_target) }?;
		let txt = unsafe { ::std::slice::from_raw_parts(txt_record, txt_len as usize) };

		Ok(ResolveData{
			interface_index: InterfaceIndex::from_raw(interface_index),
			fullname: fullname.to_string(),
			host_target: host_target.to_string(),
			port: port,
			txt: txt.into(),
		})
	});

	sender.send(data).unwrap();
}

pub fn resolve(
	interface_index: InterfaceIndex,
	name: &str,
	reg_type: &str,
	domain: &str,
	handle: &Handle
) -> io::Result<Resolve> {
	let name = cstr::CStr::from(&name)?;
	let reg_type = cstr::CStr::from(&reg_type)?;
	let domain = cstr::CStr::from(&domain)?;

	Ok(Resolve(ServiceStream::new(move |sender|
		EventedDNSService::new(
			raw::DNSService::resolve(
				0, /* no flags */
				interface_index.as_raw(),
				&name,
				&reg_type,
				&domain,
				Some(resolve_callback),
				sender as *mut c_void,
			)?,
			handle
		)
	)?))
}
