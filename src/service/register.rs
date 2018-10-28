use futures::{self,Async};
use std::os::raw::{c_void,c_char};
use std::io;
use tokio_core::reactor::{Handle,Remote};

use cstr;
use evented::EventedDNSService;
use ffi;
use interface::Interface;
use raw;
use remote::GetRemote;

type CallbackFuture = ::future::ServiceFuture<RegisterResult>;

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
#[must_use = "futures do nothing unless polled"]
pub struct Register(CallbackFuture);

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
	CallbackFuture::run_callback(context, error_code, || {
		let name = unsafe { cstr::from_cstr(name) }?;
		let reg_type = unsafe { cstr::from_cstr(reg_type) }?;
		let domain = unsafe { cstr::from_cstr(domain) }?;

		Ok(RegisterResult{
			name: name.to_string(),
			reg_type: reg_type.to_string(),
			domain: domain.to_string(),
		})
	});
}

/// Successful registration
///
/// On dropping the registration the service will be unregistered.
/// Registered [`Record`](struct.Record.html)s from this `Registration`
/// or the originating [`Register`](struct.Register.html) future will
/// keep the `Registration` alive.
pub struct Registration(EventedDNSService);

/// Optional data when registering a service; either use its default
/// value or customize it like:
///
/// ```
/// # use async_dnssd::RegisterData;
/// RegisterData {
///     txt: b"some text data",
///     .. Default::default()
/// };
/// ```
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct RegisterData<'a> {
	/// flags for registration
	pub flags: RegisterFlags,
	/// interface to register service on
	pub interface: Interface,
	/// service name, defaults to hostname
	pub name: Option<&'a str>,
	/// domain on which to advertise the service
	pub domain: Option<&'a str>,
	/// the SRV target host name, defaults to local hostname(s).
	/// Address records are NOT automatically generated for other names.
	pub host: Option<&'a str>,
	/// The TXT record rdata. Empty RDATA is treated like `b"\0"`, i.e.
	/// a TXT record with a single empty string.
	///
	/// You can use [`TxtRecord`] to create the value for this field
	/// (both [`TxtRecord::data`] and [`TxtRecord::rdata`] produce
	/// appropriate values).
	///
	/// [`TxtRecord`]: struct.TxtRecord.html
	/// [`TxtRecord::data`]: struct.TxtRecord.html#method.data
	/// [`TxtRecord::rdata`]: struct.TxtRecord.html#method.rdata
	pub txt: &'a [u8],
}

impl<'a> Default for RegisterData<'a> {
	fn default() -> Self {
		RegisterData {
			flags: RegisterFlags::default(),
			interface: Interface::default(),
			name: None,
			domain: None,
			host: None,
			txt: b"",
		}
	}
}

/// Register a service
///
/// * `reg_type`: the service type followed by the protocol, separated
///   by a dot (for example, "_ssh._tcp").  For details see
///   [`DNSServiceRegister`]
/// * `port`: The port (in native byte order) on which the service
///   accepts connections.  Pass 0 for a "placeholder" service.
/// * `data`: additional service data; `Default::default()` should be
///   fine usually.
/// * `handle`: the tokio event loop handle
///
/// See
/// [`DNSServiceRegister`](https://developer.apple.com/documentation/dnssd/1804733-dnsserviceregister).
///
/// # Example
///
/// ```no_run
/// # extern crate async_dnssd;
/// # extern crate tokio_core;
/// # use async_dnssd::register;
/// # #[deny(unused_must_use)]
/// # fn main() -> std::io::Result<()> {
/// let mut core = tokio_core::reactor::Core::new()?;
/// let handle = core.handle();
/// let registration = core.run(register("_ssh._tcp", 22, Default::default(), &handle)?)?;
/// # Ok(())
/// # }
/// ```
#[cfg_attr(feature = "cargo-clippy", allow(too_many_arguments))]
pub fn register(
	reg_type: &str,
	port: u16,
	data: RegisterData,
	handle: &Handle,
) -> io::Result<Register> {
	::init();

	let name = cstr::NullableCStr::from(&data.name)?;
	let reg_type = cstr::CStr::from(&reg_type)?;
	let domain = cstr::NullableCStr::from(&data.domain)?;
	let host = cstr::NullableCStr::from(&data.host)?;

	Ok(Register(CallbackFuture::new(handle, move |sender|
		raw::DNSService::register(
			data.flags.into(),
			data.interface.into_raw(),
			&name,
			&reg_type,
			&domain,
			&host,
			port.to_be(),
			data.txt,
			Some(register_callback),
			sender,
		)
	)?))
}

impl Register {
	/// Add a record to a registered service
	///
	/// See [`DNSServiceAddRecord`](https://developer.apple.com/documentation/dnssd/1804730-dnsserviceaddrecord)
	pub fn add_record(
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
	/// Add a record to a registered service
	///
	/// See [`DNSServiceAddRecord`](https://developer.apple.com/documentation/dnssd/1804730-dnsserviceaddrecord)
	pub fn add_record(
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
