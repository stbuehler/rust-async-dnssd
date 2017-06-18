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

flags!{QueryRecordFlags: u8: QueryRecordFlag:
	LongLivedQuery,
}

flag_mapping!{QueryRecordFlags: QueryRecordFlag => ffi::DNSServiceFlags:
	LongLivedQuery => ffi::FLAGS_LONG_LIVED_QUERY,
}

flags!{QueriedRecordFlags: u8: QueriedRecordFlag:
	MoreComing,
	Add,
}

flag_mapping!{QueriedRecordFlags: QueriedRecordFlag => ffi::DNSServiceFlags:
	MoreComing => ffi::FLAGS_MORE_COMING,
	Add => ffi::FLAGS_ADD,
}

pub struct QueryRecord(ServiceStream<QueryRecordData>);

impl futures::Stream for QueryRecord {
	type Item = QueryRecordData;
	type Error = io::Error;

	fn poll(&mut self) -> Result<Async<Option<Self::Item>>, Self::Error> {
		self.0.poll()
	}
}

impl GetRemote for QueryRecord {
	fn remote(&self) -> &Remote {
		self.0.remote()
	}
}

#[derive(Clone,PartialEq,Eq,PartialOrd,Ord,Hash,Debug)]
pub struct QueryRecordData{
	pub flags: QueriedRecordFlags,
	pub interface_index: InterfaceIndex,
	pub fullname: String,
	pub rr_type: u16,
	pub rr_class: u16,
	pub rdata: Vec<u8>,
	pub ttl: u32,
}

extern "C" fn query_record_callback(
	_sd_ref: ffi::DNSServiceRef,
	flags: ffi::DNSServiceFlags,
	interface_index: u32,
	error_code: ffi::DNSServiceErrorType,
	fullname: *const c_char,
	rr_type: u16,
	rr_class: u16,
	rd_len: u16,
	rdata: *const u8,
	ttl: u32,
	context: *mut c_void
) {
	let sender = context as *mut mpsc::UnboundedSender<io::Result<QueryRecordData>>;
	let sender : &mpsc::UnboundedSender<io::Result<QueryRecordData>> = unsafe { &*sender };

	let data = Error::from(error_code).map_err(io::Error::from).and_then(|_| {
		let fullname = unsafe { cstr::from_cstr(fullname) }?;
		let rdata = unsafe { ::std::slice::from_raw_parts(rdata, rd_len as usize) };

		Ok(QueryRecordData{
			flags: QueriedRecordFlags::from(flags),
			interface_index: InterfaceIndex::from_raw(interface_index),
			fullname: fullname.to_string(),
			rr_type: rr_type,
			rr_class: rr_class,
			rdata: rdata.into(),
			ttl: ttl,
		})
	});

	sender.send(data).unwrap();
}

pub fn query_record(
	flags: QueryRecordFlags,
	interface_index: InterfaceIndex,
	fullname: &str,
	rr_type: u16,
	rr_class: u16,
	handle: &Handle
) -> io::Result<QueryRecord> {
	let fullname = cstr::CStr::from(&fullname)?;

	Ok(QueryRecord(ServiceStream::new(move |sender|
		EventedDNSService::new(
			raw::DNSService::query_record(
				flags.into(),
				interface_index.as_raw(),
				&fullname,
				rr_type,
				rr_class,
				Some(query_record_callback),
				sender as *mut c_void,
			)?,
			handle
		)
	)?))
}
