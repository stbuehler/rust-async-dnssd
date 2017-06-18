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

flags!{BrowsedFlags: u8: BrowsedFlag:
	MoreComing,
	Add,
}

flag_mapping!{BrowsedFlags: BrowsedFlag => ffi::DNSServiceFlags:
	MoreComing => ffi::FLAGS_MORE_COMING,
	Add => ffi::FLAGS_ADD,
}


pub struct Browse(ServiceStream<BrowseData>);

impl futures::Stream for Browse {
	type Item = BrowseData;
	type Error = io::Error;

	fn poll(&mut self) -> Result<Async<Option<Self::Item>>, Self::Error> {
		self.0.poll()
	}
}

impl GetRemote for Browse {
	fn remote(&self) -> &Remote {
		self.0.remote()
	}
}

#[derive(Clone,PartialEq,Eq,PartialOrd,Ord,Hash,Debug)]
pub struct BrowseData{
	pub flags: BrowsedFlags,
	pub interface_index: InterfaceIndex,
	pub service_name: String,
	pub reg_type: String,
	pub reply_domain: String,
}

impl BrowseData {
	pub fn resolve(&self, handle: &Handle) -> io::Result<::Resolve> {
		::resolve(
			self.interface_index,
			&self.service_name,
			&self.reg_type,
			&self.reply_domain,
			handle
		)
	}
}

extern "C" fn browse_callback(
	_sd_ref: ffi::DNSServiceRef,
	flags: ffi::DNSServiceFlags,
	interface_index: u32,
	error_code: ffi::DNSServiceErrorType,
	service_name: *const c_char,
	reg_type: *const c_char,
	reply_domain: *const c_char,
	context: *mut c_void
) {
	let sender = context as *mut mpsc::UnboundedSender<io::Result<BrowseData>>;
	let sender : &mpsc::UnboundedSender<io::Result<BrowseData>> = unsafe { &*sender };

	let data = Error::from(error_code).map_err(io::Error::from).and_then(|_| {
		let service_name = unsafe { cstr::from_cstr(service_name) }?;
		let reg_type = unsafe { cstr::from_cstr(reg_type) }?;
		let reply_domain = unsafe { cstr::from_cstr(reply_domain) }?;

		Ok(BrowseData{
			flags: BrowsedFlags::from(flags),
			interface_index: InterfaceIndex::from_raw(interface_index),
			service_name: service_name.to_string(),
			reg_type: reg_type.to_string(),
			reply_domain: reply_domain.to_string(),
		})
	});

	sender.send(data).unwrap();
}

pub fn browse(
	interface_index: InterfaceIndex,
	reg_type: &str,
	domain: Option<&str>,
	handle: &Handle
) -> io::Result<Browse> {
	let reg_type = cstr::CStr::from(&reg_type)?;
	let domain = cstr::NullableCStr::from(&domain)?;

	Ok(Browse(ServiceStream::new(move |sender|
		EventedDNSService::new(
			raw::DNSService::browse(
				0, /* no flags */
				interface_index.as_raw(),
				&reg_type,
				&domain,
				Some(browse_callback),
				sender as *mut c_void,
			)?,
			handle
		)
	)?))
}
