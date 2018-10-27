use futures::{self,Async,Future};
use std::os::raw::{c_void};
use std::io;
use std::rc::Rc;
use tokio_core::reactor::{Handle,Remote};

use cstr;
use evented::EventedDNSService;
use ffi;
use interface::Interface;
use raw;
use remote::GetRemote;

type CallbackFuture = ::future::ServiceFutureSingle<RegisterRecordResult>;

/// Connection to register records with
pub struct Connection(Rc<EventedDNSService>);

impl GetRemote for Connection {
	fn remote(&self) -> &Remote {
		self.0.remote()
	}
}

/// Create [`Connection`](struct.Connection.html) to register records
/// with
///
/// See [`DNSServiceCreateConnection`](https://developer.apple.com/documentation/dnssd/1804724-dnsservicecreateconnection).
pub fn connect(handle: &Handle) -> io::Result<Connection> {
	let con = raw::DNSService::create_connection()?;
	Ok(Connection(Rc::new(
		EventedDNSService::new(con, handle)?
	)))
}

/// Set of [`RegisterRecordFlag`](enum.RegisterRecordFlag.html)s
///
/// Flags and sets can be combined with bitor (`|`), and bitand (`&`)
/// can be used to test whether a flag is part of a set.
#[derive(Clone,Copy,PartialEq,Eq,PartialOrd,Ord,Hash)]
pub struct RegisterRecordFlags(u8);

/// Flags used to register a record
#[derive(Clone,Copy,PartialEq,Eq,PartialOrd,Ord,Hash,Debug)]
#[repr(u8)]
pub enum RegisterRecordFlag {
	/// Indicates there might me multiple records with the given name, type and class.
	///
	/// See [`kDNSServiceFlagsShared`](https://developer.apple.com/documentation/dnssd/1823436-anonymous/kdnsserviceflagsshared).
	Shared = 0,

	/// Indicates the records with the given name, type and class is unique.
	///
	/// See [`kDNSServiceFlagsUnique`](https://developer.apple.com/documentation/dnssd/1823436-anonymous/kdnsserviceflagsunique).
	Unique,
}

flags_ops!{RegisterRecordFlags: u8: RegisterRecordFlag:
	Shared,
	Unique,
}

flag_mapping!{RegisterRecordFlags: RegisterRecordFlag => ffi::DNSServiceFlags:
	Shared => ffi::FLAGS_SHARED,
	Unique => ffi::FLAGS_UNIQUE,
}

/// Pending record registration
///
/// Becomes invalid when the future completes; use the returned
/// [`Record`](struct.Record.html) instead.
// the future gets canceled by dropping the record; must
// not drop the future without dropping the record.
pub struct RegisterRecord(CallbackFuture, Option<raw::DNSRecord>);

impl futures::Future for RegisterRecord {
	type Item = ::Record;
	type Error = io::Error;

	fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
		match self.0.poll() {
			Ok(Async::Ready(RegisterRecordResult)) => Ok(Async::Ready(
				super::new_record(self.1.take().unwrap())
			)),
			Ok(Async::NotReady) => Ok(Async::NotReady),
			Err(e) => Err(e),
		}
	}
}

impl GetRemote for RegisterRecord {
	fn remote(&self) -> &Remote {
		self.0.remote()
	}
}

#[derive(Clone,PartialEq,Eq,PartialOrd,Ord,Hash,Debug)]
struct RegisterRecordResult;

extern "C" fn register_record_callback(
	_sd_ref: ffi::DNSServiceRef,
	_record_ref: ffi::DNSRecordRef,
	_flags: ffi::DNSServiceFlags,
	error_code: ffi::DNSServiceErrorType,
	context: *mut c_void
) {
	CallbackFuture::run_callback(context, error_code, || {
		Ok(RegisterRecordResult)
	});
}

impl Connection {
	/// Register record on interface with given name, type, class, rdata
	/// and ttl
	///
	/// See [`DNSServiceRegisterRecord`](https://developer.apple.com/documentation/dnssd/1804727-dnsserviceregisterrecord).
	pub fn register_record(
		&self,
		flags: RegisterRecordFlags,
		interface: Interface,
		fullname: &str,
		rr_type: u16,
		rr_class: u16,
		rdata: &[u8],
		ttl: u32
	) -> io::Result<RegisterRecord> {
		let fullname = cstr::CStr::from(&fullname)?;

		let (serv, record) = CallbackFuture::new(self.0.clone(), move |sender|
			self.0.service().register_record(
				flags.into(),
				interface.into_raw(),
				&fullname,
				rr_type,
				rr_class,
				rdata,
				ttl,
				Some(register_record_callback),
				sender,
			)
		)?;

		Ok(RegisterRecord(serv, Some(record)))
	}
}

impl RegisterRecord {
	fn record(&self) -> &raw::DNSRecord {
		self.1.as_ref().expect("RegisterRecord future is done")
	}

	/// Type of the record
	///
	/// # Panics
	///
	/// Panics after the future completed.  Use the returned
	/// [`Record`](struct.Record.html) instead.
	pub fn rr_type(&self) -> u16 {
		self.record().rr_type()
	}

	/// Update record
	///
	/// Cannot change type or class of record.
	///
	/// # Panics
	///
	/// Panics after the future completed.  Use the returned
	/// [`Record`](struct.Record.html) instead.
	///
	/// See [`DNSServiceUpdateRecord`](https://developer.apple.com/documentation/dnssd/1804739-dnsserviceupdaterecord).
	pub fn update_record(
		&self,
		rdata: &[u8],
		ttl: u32
	) -> io::Result<()> {
		self.record().update_record(
			0, /* no flags */
			rdata,
			ttl
		)?;
		Ok(())
	}

	/// Keep record for as long as the underlying connection lives.
	///
	/// Keep the a handle to the underlying connection (either the
	/// [`Connection`](struct.Connection.html) or some other record from
	/// the same `Connection`) alive.
	///
	/// Due to some implementation detail the underlying connection
	/// might live until this future successfully completes.
	///
	/// # Panics
	///
	/// Panics after the future completed.  Use the returned
	/// [`Record`](struct.Record.html) instead.
	// - implementation detail: this drives the future to continuation,
	//   it is not possible to drop the (shared) underlying service
	//   before. instead we could store the callback context with the
	//   underyling service, and drop it either when dropping the
	//   service or the callback was called.
	pub fn keep(self, handle: &Handle) {
		let (fut, rec) = (self.0, self.1.expect("RegisterRecord future is done"));
		// drive future to continuation, ignore errors
		handle.spawn(fut.then(|_| Ok(())));
		rec.keep();
	}
}
