use futures::sync::mpsc;
use futures::{self,Async};
use std::os::raw::{c_void,c_char};
use std::io;
use tokio_core::reactor::{Handle,Remote};

use cstr;
use error::Error;
use evented::EventedDNSService;
use ffi;
use interface::Interface;
use raw;
use remote::GetRemote;
use future::ServiceFuture;

/// Set of [`RegisterFlag`](enum.RegisterFlag.html)s
///
/// Flags and sets can be combined with bitor (`|`), and bitand (`&`)
/// can be used to test whether a flag is part of a set.
#[derive(Clone,Copy,PartialEq,Eq,PartialOrd,Ord,Hash)]
pub struct RegisterFlags(u8);

/// Flags used to register service
#[derive(Clone,Copy,PartialEq,Eq,PartialOrd,Ord,Hash,Debug)]
#[repr(u8)]
pub enum RegisterFlag {
	/// Indicates a name conflict should not get handled automatically.
	///
	/// See [`kDNSServiceFlagsNoAutoRename`](https://developer.apple.com/documentation/dnssd/1823436-anonymous/kdnsserviceflagsnoautorename).
	NoAutoRename = 0,

	/// Indicates there might me multiple records with the given name, type and class.
	///
	/// See [`kDNSServiceFlagsShared`](https://developer.apple.com/documentation/dnssd/1823436-anonymous/kdnsserviceflagsshared).
	Shared,

	/// Indicates the records with the given name, type and class is unique.
	///
	/// See [`kDNSServiceFlagsUnique`](https://developer.apple.com/documentation/dnssd/1823436-anonymous/kdnsserviceflagsunique).
	Unique,
}

flags_ops!{RegisterFlags: u8: RegisterFlag:
	NoAutoRename,
	Shared,
	Unique,
}

flag_mapping!{RegisterFlags: RegisterFlag => ffi::DNSServiceFlags:
	NoAutoRename => ffi::FLAGS_NO_AUTO_RENAME,
	Shared => ffi::FLAGS_SHARED,
	Unique => ffi::FLAGS_UNIQUE,
}

/// Pending registration
///
/// Becomes invalid when the future completes; use the returned
/// [`Registration`](struct.Registration.html) instead.
pub struct Register(ServiceFuture<RegisterResult>);

impl futures::Future for Register {
	type Item = (Registration, RegisterResult);
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

/// Service registration result
///
/// See [`DNSServiceRegisterReply`](https://developer.apple.com/documentation/dnssd/dnsserviceregisterreply).
#[derive(Clone,PartialEq,Eq,PartialOrd,Ord,Hash,Debug)]
pub struct RegisterResult{
	/// if [`NoAutoRename`](enum.RegisterFlag.html#variant.NoAutoRename)
	/// was set this is the original name, otherwise it might be
	/// different.
	pub name: String,
	///
	pub reg_type: String,
	///
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
	let sender = context as *mut mpsc::UnboundedSender<io::Result<RegisterResult>>;
	let sender : &mpsc::UnboundedSender<io::Result<RegisterResult>> = unsafe { &*sender };

	let data = Error::from(error_code).map_err(io::Error::from).and_then(|_| {
		let name = unsafe { cstr::from_cstr(name) }?;
		let reg_type = unsafe { cstr::from_cstr(reg_type) }?;
		let domain = unsafe { cstr::from_cstr(domain) }?;

		Ok(RegisterResult{
			name: name.to_string(),
			reg_type: reg_type.to_string(),
			domain: domain.to_string(),
		})
	});

	sender.unbounded_send(data).unwrap();
}

/// Successful registration
///
/// On dropping the registration the service will be unregistered.
/// Registered [`Record`](struct.Record.html)s from this `Registration`
/// or the originating [`Register`](struct.Register.html) future will
/// keep the `Registration` alive.
pub struct Registration(EventedDNSService);

/// Registers a service
///
/// See [`DNSServiceRegister`](https://developer.apple.com/documentation/dnssd/1804733-dnsserviceregister).
pub fn register(
	flags: RegisterFlags,
	interface: Interface,
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
				interface.into_raw(),
				&name,
				&reg_type,
				&domain,
				&host,
				port.to_be(),
				txt,
				Some(register_callback),
				sender as *mut c_void,
			)?,
			handle
		)
	)?))
}

impl Register {
	/// See [`DNSServiceAddRecord`](https://developer.apple.com/documentation/dnssd/1804730-dnsserviceaddrecord)
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

	/// Get [`Record`](struct.Record.html) handle for default TXT record
	/// associated with the service registration (e.g. to update it).
	///
	/// [`Record::keep`](struct.Record.html#method.keep) doesn't do
	/// anything useful on that handle.
	pub fn get_default_txt_record(&self) -> ::Record {
		super::new_record(self.0.service().get_default_txt_record())
	}
}

impl Registration {
	/// See [`DNSServiceAddRecord`](https://developer.apple.com/documentation/dnssd/1804730-dnsserviceaddrecord)
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

	/// Get [`Record`](struct.Record.html) handle for default TXT record
	/// associated with the service registration (e.g. to update it).
	///
	/// [`Record::keep`](struct.Record.html#method.keep) doesn't do
	/// anything useful on that handle.
	pub fn get_default_txt_record(&self) -> ::Record {
		super::new_record(self.0.service().get_default_txt_record())
	}
}
