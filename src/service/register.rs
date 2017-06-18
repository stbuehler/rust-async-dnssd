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
use future::ServiceFuture;

flags!{RegisterFlags: u8: RegisterFlag:
	NoAutoRename,
	Shared,
	Unique,
}

flag_mapping!{RegisterFlags: RegisterFlag => ffi::DNSServiceFlags:
	NoAutoRename => ffi::FLAGS_NO_AUTO_RENAME,
	Shared => ffi::FLAGS_SHARED,
	Unique => ffi::FLAGS_UNIQUE,
}

pub struct Register(ServiceFuture<RegisterData>);

impl futures::Future for Register {
	type Item = (Registration, RegisterData);
	type Error = io::Error;

	fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
		match self.0.poll() {
			Ok(Async::Ready((service, item))) => Ok(Async::Ready((
				Registration(service),
				item
			))),
			Ok(Async::NotReady) => Ok(Async::NotReady),
			Err(e) => Err(e),
		}
	}
}

impl GetRemote for Register {
	fn remote(&self) -> &Remote {
		self.0.remote()
	}
}

#[derive(Clone,PartialEq,Eq,PartialOrd,Ord,Hash,Debug)]
pub struct RegisterData{
	pub name: String,
	pub reg_type: String,
	pub domain: String,
}

extern "C" fn register_callback(
	_sd_ref: ffi::DNSServiceRef,
	_flags: ffi::DNSServiceFlags,
	error_code: ffi::DNSServiceErrorType,
	name: *const c_char,
	reg_type: *const c_char,
	domain: *const c_char,
	context: *mut c_void
) {
	let sender = context as *mut mpsc::UnboundedSender<io::Result<RegisterData>>;
	let sender : &mpsc::UnboundedSender<io::Result<RegisterData>> = unsafe { &*sender };

	let data = Error::from(error_code).map_err(io::Error::from).and_then(|_| {
		let name = unsafe { cstr::from_cstr(name) }?;
		let reg_type = unsafe { cstr::from_cstr(reg_type) }?;
		let domain = unsafe { cstr::from_cstr(domain) }?;

		Ok(RegisterData{
			name: name.to_string(),
			reg_type: reg_type.to_string(),
			domain: domain.to_string(),
		})
	});

	sender.send(data).unwrap();
}

pub struct Registration(EventedDNSService);

pub fn register(
	flags: RegisterFlags,
	interface_index: InterfaceIndex,
	name: Option<&str>,
	reg_type: &str,
	domain: Option<&str>,
	host: Option<&str>,
	port: u16,
	txt: &[u8],
	handle: &Handle
) -> io::Result<Register> {
	let name = cstr::NullableCStr::from(&name)?;
	let reg_type = cstr::CStr::from(&reg_type)?;
	let domain = cstr::NullableCStr::from(&domain)?;
	let host = cstr::NullableCStr::from(&host)?;

	Ok(Register(ServiceFuture::new(move |sender|
		EventedDNSService::new(
			raw::DNSService::register(
				flags.into(),
				interface_index.as_raw(),
				&name,
				&reg_type,
				&domain,
				&host,
				port,
				txt,
				Some(register_callback),
				sender as *mut c_void,
			)?,
			handle
		)
	)?))
}

impl Register {
	pub fn add_raw_record(
		&self,
		rr_type: u16,
		rdata: &[u8],
		ttl: u32
	) -> io::Result<::Record> {
		Ok(super::new_record(self.0.service().add_record(
			0, /* no flags */
			rr_type,
			rdata,
			ttl
		)?))
	}

	pub fn get_default_txt_record(&self) -> ::Record {
		super::new_record(self.0.service().get_default_txt_record())
	}
}

impl Registration {
	pub fn add_raw_record(
		&self,
		rr_type: u16,
		rdata: &[u8],
		ttl: u32
	) -> io::Result<::Record> {
		Ok(super::new_record(self.0.service().add_record(
			0, /* no flags */
			rr_type,
			rdata,
			ttl
		)?))
	}

	pub fn get_default_txt_record(&self) -> ::Record {
		super::new_record(self.0.service().get_default_txt_record())
	}
}
