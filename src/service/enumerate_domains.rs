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

#[derive(Clone,Copy,PartialEq,Eq,PartialOrd,Ord,Hash,Debug)]
pub enum Enumerate {
	BrowseDomains,
	RegistrationDomains,
}

impl Into<ffi::DNSServiceFlags> for Enumerate {
	fn into(self) -> ffi::DNSServiceFlags {
		match self {
			Enumerate::BrowseDomains => ffi::FLAGS_BROWSE_DOMAINS,
			Enumerate::RegistrationDomains => ffi::FLAGS_REGISTRATION_DOMAINS,
		}
	}
}

flags!{EnumeratedFlags: u8: EnumeratedFlag:
	MoreComing,
	Add,
	Default,
}

flag_mapping!{EnumeratedFlags: EnumeratedFlag => ffi::DNSServiceFlags:
	MoreComing => ffi::FLAGS_MORE_COMING,
	Add => ffi::FLAGS_ADD,
	Default => ffi::FLAGS_DEFAULT,
}


pub struct EnumerateDomains(ServiceStream<EnumerateData>);

impl futures::Stream for EnumerateDomains {
	type Item = EnumerateData;
	type Error = io::Error;

	fn poll(&mut self) -> Result<Async<Option<Self::Item>>, Self::Error> {
		self.0.poll()
	}
}

impl GetRemote for EnumerateDomains {
	fn remote(&self) -> &Remote {
		self.0.remote()
	}
}

#[derive(Clone,PartialEq,Eq,PartialOrd,Ord,Hash,Debug)]
pub struct EnumerateData{
	pub flags: EnumeratedFlags,
	pub interface_index: InterfaceIndex,
	pub reply_domain: String,
}

extern "C" fn enumerate_callback(
	_sd_ref: ffi::DNSServiceRef,
	flags: ffi::DNSServiceFlags,
	interface_index: u32,
	error_code: ffi::DNSServiceErrorType,
	reply_domain: *const c_char,
	context: *mut c_void
) {
	let sender = context as *mut mpsc::UnboundedSender<io::Result<EnumerateData>>;
	let sender : &mpsc::UnboundedSender<io::Result<EnumerateData>> = unsafe { &*sender };

	let data = Error::from(error_code).map_err(io::Error::from).and_then(|_| {
		let reply_domain = unsafe { cstr::from_cstr(reply_domain) }?;

		Ok(EnumerateData{
			flags: EnumeratedFlags::from(flags),
			interface_index: InterfaceIndex::from_raw(interface_index),
			reply_domain: reply_domain.to_string(),
		})
	});

	sender.send(data).unwrap();
}

pub fn enumerate_domains(enumerate: Enumerate, interface_index: InterfaceIndex, handle: &Handle) -> io::Result<EnumerateDomains> {
	Ok(EnumerateDomains(ServiceStream::new(move |sender|
		EventedDNSService::new(
			raw::DNSService::enumerate_domains(
				enumerate.into(),
				interface_index.as_raw(),
				Some(enumerate_callback),
				sender as *mut c_void,
			)?,
			handle
		)
	)?))
}
