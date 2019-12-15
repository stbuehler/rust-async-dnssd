use bitflags::bitflags;
use futures::{
	self,
	Async,
	Future,
};
use std::{
	io,
	os::raw::c_void,
	rc::Rc,
};

use crate::{
	cstr,
	dns_consts::{
		Class,
		Type,
	},
	evented::EventedDNSService,
	ffi,
	interface::Interface,
	raw,
};

type CallbackFuture = crate::future::ServiceFutureSingle<RegisterRecordResult>;

/// Connection to register records with
pub struct Connection(Rc<EventedDNSService>);

/// Create [`Connection`](struct.Connection.html) to register records
/// with
///
/// See [`DNSServiceCreateConnection`](https://developer.apple.com/documentation/dnssd/1804724-dnsservicecreateconnection).
pub fn connect() -> io::Result<Connection> {
	crate::init();

	let con = raw::DNSService::create_connection()?;
	Ok(Connection(Rc::new(EventedDNSService::new(con)?)))
}

bitflags! {
	/// Flags used to register a record
	#[derive(Default)]
	pub struct RegisterRecordFlags: ffi::DNSServiceFlags {
		/// Indicates there might me multiple records with the given name, type and class.
		///
		/// See [`kDNSServiceFlagsShared`](https://developer.apple.com/documentation/dnssd/1823436-anonymous/kdnsserviceflagsshared).
		const SHARED = ffi::FLAGS_SHARED;

		/// Indicates the records with the given name, type and class is unique.
		///
		/// See [`kDNSServiceFlagsUnique`](https://developer.apple.com/documentation/dnssd/1823436-anonymous/kdnsserviceflagsunique).
		const UNIQUE = ffi::FLAGS_UNIQUE;
	}
}

/// Pending record registration
///
/// Becomes invalid when the future completes; use the returned
/// [`Record`](struct.Record.html) instead.
// the future gets canceled by dropping the record; must
// not drop the future without dropping the record.
#[must_use = "futures do nothing unless polled"]
pub struct RegisterRecord(CallbackFuture, Option<crate::Record>);

impl Future for RegisterRecord {
	type Error = io::Error;
	type Item = crate::Record;

	fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
		match self.0.poll() {
			Ok(Async::Ready(RegisterRecordResult)) => Ok(Async::Ready(self.1.take().unwrap())),
			Ok(Async::NotReady) => Ok(Async::NotReady),
			Err(e) => Err(e),
		}
	}
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
struct RegisterRecordResult;

extern "C" fn register_record_callback(
	_sd_ref: ffi::DNSServiceRef,
	_record_ref: ffi::DNSRecordRef,
	_flags: ffi::DNSServiceFlags,
	error_code: ffi::DNSServiceErrorType,
	context: *mut c_void,
) {
	CallbackFuture::run_callback(context, error_code, || Ok(RegisterRecordResult));
}

/// Optional data when registering a record; either use its default
/// value or customize it like:
///
/// ```
/// # use async_dnssd::RegisterRecordData;
/// RegisterRecordData {
///     ttl: 60,
///     ..Default::default()
/// };
/// ```
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct RegisterRecordData {
	/// flags for registration
	pub flags: RegisterRecordFlags,
	/// interface to register record on
	pub interface: Interface,
	/// class of the resource record (default: `IN`)
	pub rr_class: Class,
	/// time to live of the resource record in seconds (passing 0 will
	/// select a sensible default)
	pub ttl: u32,
}

impl Default for RegisterRecordData {
	fn default() -> Self {
		RegisterRecordData {
			flags: RegisterRecordFlags::default(),
			interface: Interface::default(),
			rr_class: Class::IN,
			ttl: 0,
		}
	}
}

impl Connection {
	/// Register record on interface with given name, type, class, rdata
	/// and ttl
	///
	/// See [`DNSServiceRegisterRecord`](https://developer.apple.com/documentation/dnssd/1804727-dnsserviceregisterrecord).
	pub fn register_record_extended(
		&self,
		fullname: &str,
		rr_type: Type,
		rdata: &[u8],
		data: RegisterRecordData,
	) -> io::Result<RegisterRecord> {
		let fullname = cstr::CStr::from(&fullname)?;

		let (serv, record) = CallbackFuture::new(self.0.clone(), move |sender| {
			self.0.service().register_record(
				data.flags.bits(),
				data.interface.into_raw(),
				&fullname,
				rr_type,
				data.rr_class,
				rdata,
				data.ttl,
				Some(register_record_callback),
				sender,
			)
		})?;

		Ok(RegisterRecord(serv, Some(record.into())))
	}

	/// Register record on interface with given name, type, class, rdata
	/// and ttl
	///
	/// Uses [`register_record_extended`] with default [`RegisterRecordData`].
	///
	/// See [`DNSServiceRegisterRecord`](https://developer.apple.com/documentation/dnssd/1804727-dnsserviceregisterrecord).
	///
	/// [`register_record_extended`]: fn.register_record_extended.html
	/// [`RegisterRecordData`]: struct.RegisterRecordData.html
	pub fn register_record(
		&self,
		fullname: &str,
		rr_type: Type,
		rdata: &[u8],
	) -> io::Result<RegisterRecord> {
		self.register_record_extended(fullname, rr_type, rdata, RegisterRecordData::default())
	}
}

impl RegisterRecord {
	fn record(&self) -> &crate::Record {
		self.1.as_ref().expect("RegisterRecord future is done")
	}

	/// Type of the record
	///
	/// # Panics
	///
	/// Panics after the future completed.  Use the returned
	/// [`Record`](struct.Record.html) instead.
	pub fn rr_type(&self) -> Type {
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
	pub fn update_record(&self, rdata: &[u8], ttl: u32) -> io::Result<()> {
		self.record().update_record(rdata, ttl)
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
	pub fn keep(self) {
		let (fut, rec) = (self.0, self.1.expect("RegisterRecord future is done"));
		// drive future to continuation, ignore errors
		tokio::runtime::current_thread::spawn(fut.then(|_| Ok(())));
		rec.keep();
	}
}
