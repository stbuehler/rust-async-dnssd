use futures::sync::mpsc;
use futures::{self,Async,Future};
use std::os::raw::{c_void};
use std::io;
use std::rc::Rc;
use tokio_core::reactor::{Handle,Remote};

use cstr;
use error::Error;
use evented::EventedDNSService;
use ffi;
use interface_index::InterfaceIndex;
use raw;
use remote::GetRemote;
use future::ServiceFutureSingle;

pub struct Connection(Rc<EventedDNSService>);

impl GetRemote for Connection {
	fn remote(&self) -> &Remote {
		self.0.remote()
	}
}

pub fn connect(handle: &Handle) -> io::Result<Connection> {
	let con = raw::DNSService::create_connection()?;
	Ok(Connection(Rc::new(
		EventedDNSService::new(con, handle)?
	)))
}

flags!{RegisterRecordFlags: u8: RegisterRecordFlag:
	Shared,
	Unique,
}

flag_mapping!{RegisterRecordFlags: RegisterRecordFlag => ffi::DNSServiceFlags:
	Shared => ffi::FLAGS_SHARED,
	Unique => ffi::FLAGS_UNIQUE,
}

// the future gets canceled by dropping the record; must
// not drop the future without dropping the record.
pub struct RegisterRecord(ServiceFutureSingle<RegisterRecordData>, Option<raw::DNSRecord>);

impl futures::Future for RegisterRecord {
	type Item = ::Record;
	type Error = io::Error;

	fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
		match self.0.poll() {
			Ok(Async::Ready(RegisterRecordData)) => Ok(Async::Ready(
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
struct RegisterRecordData;

extern "C" fn register_record_callback(
	_sd_ref: ffi::DNSServiceRef,
	_record_ref: ffi::DNSRecordRef,
	_flags: ffi::DNSServiceFlags,
	error_code: ffi::DNSServiceErrorType,
	context: *mut c_void
) {
	let sender = context as *mut mpsc::UnboundedSender<io::Result<RegisterRecordData>>;
	let sender : &mpsc::UnboundedSender<io::Result<RegisterRecordData>> = unsafe { &*sender };

	let data = Error::from(error_code).map_err(io::Error::from).and_then(|_| {
		Ok(RegisterRecordData)
	});

	sender.send(data).unwrap();
}

impl Connection {
	pub fn register_raw_record(
		&self,
		flags: RegisterRecordFlags,
		interface_index: InterfaceIndex,
		fullname: &str,
		rr_type: u16,
		rr_class: u16,
		rdata: &[u8],
		ttl: u32
	) -> io::Result<RegisterRecord> {
		let fullname = cstr::CStr::from(&fullname)?;

		let (serv, record) = ServiceFutureSingle::new(self.0.clone(), move |sender|
			Ok(self.0.service().register_record(
				flags.into(),
				interface_index.as_raw(),
				&fullname,
				rr_type,
				rr_class,
				rdata,
				ttl,
				Some(register_record_callback),
				sender as *mut c_void,
			)?)
		)?;

		Ok(RegisterRecord(serv, Some(record)))
	}
}

impl RegisterRecord {
	fn record(&self) -> &raw::DNSRecord {
		self.1.as_ref().expect("RegisterRecord future is done")
	}

	pub fn rr_type(&self) -> u16 {
		self.record().rr_type()
	}

	pub fn update_raw_record(
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

	// keep "forever" (until service is dropped; but service is not
	// dropped before the record future is completed)
	pub fn keep(self, handle: &Handle) {
		let (fut, rec) = (self.0, self.1.expect("RegisterRecord future is done"));
		// drive future to continuation, ignore errors
		handle.spawn(fut.then(|_| Ok(())));
		rec.keep();
	}
}
