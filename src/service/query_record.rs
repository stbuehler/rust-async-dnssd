use futures::{
	self,
	Async,
};
use std::{
	io,
	os::raw::{
		c_char,
		c_void,
	},
};
use tokio_core::reactor::{
	Handle,
	Remote,
};

use cstr;
use dns_consts::{
	Class,
	Type,
};
use ffi;
use interface::Interface;
use raw;
use remote::GetRemote;

type CallbackStream = ::stream::ServiceStream<QueryRecordResult>;

bitflags! {
	/// Flags used to query for a record
	#[derive(Default)]
	pub struct QueryRecordFlags: ffi::DNSServiceFlags {
		/// long-lived unicast query
		///
		/// See [`kDNSServiceFlagsLongLivedQuery`](https://developer.apple.com/documentation/dnssd/1823436-anonymous/kdnsserviceflagslonglivedquery).
		const LONG_LIVED_QUERY = ffi::FLAGS_LONG_LIVED_QUERY;
	}
}

bitflags! {
	/// Flags for [`QueryRecordResult`](struct.QueryRecordResult.html)
	#[derive(Default)]
	pub struct QueriedRecordFlags: ffi::DNSServiceFlags {
		/// Indicates at least one more result is pending in the queue.  If
		/// not set there still might be more results coming in the future.
		///
		/// See [`kDNSServiceFlagsMoreComing`](https://developer.apple.com/documentation/dnssd/1823436-anonymous/kdnsserviceflagsmorecoming).
		const MORE_COMING = ffi::FLAGS_MORE_COMING;

		/// Indicates the result is new.  If not set indicates the result
		/// was removed.
		///
		/// See [`kDNSServiceFlagsAdd`](https://developer.apple.com/documentation/dnssd/1823436-anonymous/kdnsserviceflagsadd).
		const ADD = ffi::FLAGS_ADD;
	}
}

/// Pending query
#[must_use = "streams do nothing unless polled"]
pub struct QueryRecord(CallbackStream);

impl futures::Stream for QueryRecord {
	type Error = io::Error;
	type Item = QueryRecordResult;

	fn poll(&mut self) -> Result<Async<Option<Self::Item>>, Self::Error> {
		self.0.poll()
	}
}

impl GetRemote for QueryRecord {
	fn remote(&self) -> &Remote {
		self.0.remote()
	}
}

/// Query result
///
/// See [`DNSServiceQueryRecordReply`](https://developer.apple.com/documentation/dnssd/dnsservicequeryrecordreply).
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct QueryRecordResult {
	/// flags
	pub flags: QueriedRecordFlags,
	/// interface the record was found on
	pub interface: Interface,
	/// name of record
	pub fullname: String,
	/// type of record
	pub rr_type: Type,
	/// class of record
	pub rr_class: Class,
	/// wire RDATA of record
	pub rdata: Vec<u8>,
	/// TTL (time to live) of record
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
	context: *mut c_void,
) {
	CallbackStream::run_callback(context, error_code, || {
		let fullname = unsafe { cstr::from_cstr(fullname) }?;
		let rdata =
			unsafe { ::std::slice::from_raw_parts(rdata, rd_len as usize) };

		Ok(QueryRecordResult {
			flags: QueriedRecordFlags::from_bits_truncate(flags),
			interface: Interface::from_raw(interface_index),
			fullname: fullname.to_string(),
			rr_type: Type(rr_type),
			rr_class: Class(rr_class),
			rdata: rdata.into(),
			ttl,
		})
	});
}

/// Optional data when querying for a record; either use its default
/// value or customize it like:
///
/// ```
/// # use async_dnssd::QueryRecordData;
/// # use async_dnssd::QueryRecordFlags;
/// QueryRecordData {
/// 	flags: QueryRecordFlags::LONG_LIVED_QUERY,
/// 	..Default::default()
/// 	};
/// ```
pub struct QueryRecordData {
	/// flags for query
	pub flags: QueryRecordFlags,
	/// interface to query records on
	pub interface: Interface,
	/// class of the resource record (default: `IN`)
	pub rr_class: Class,
}

impl Default for QueryRecordData {
	fn default() -> Self {
		QueryRecordData {
			flags: QueryRecordFlags::default(),
			interface: Interface::default(),
			rr_class: Class::IN,
		}
	}
}

/// Query for an arbitrary DNS record
///
/// See [`DNSServiceQueryRecord`](https://developer.apple.com/documentation/dnssd/1804747-dnsservicequeryrecord).
pub fn query_record_extended(
	fullname: &str,
	rr_type: Type,
	data: QueryRecordData,
	handle: &Handle,
) -> io::Result<QueryRecord> {
	::init();

	let fullname = cstr::CStr::from(&fullname)?;

	Ok(QueryRecord(CallbackStream::new(handle, move |sender| {
		raw::DNSService::query_record(
			data.flags.bits(),
			data.interface.into_raw(),
			&fullname,
			rr_type,
			data.rr_class,
			Some(query_record_callback),
			sender,
		)
	})?))
}

/// Query for an arbitrary DNS record
///
/// Uses [`query_record_extended`] with default [`QueryRecordData`].
///
/// See [`DNSServiceQueryRecord`](https://developer.apple.com/documentation/dnssd/1804747-dnsservicequeryrecord).
///
/// [`query_record_extended`]: fn.query_record_extended.html
/// [`QueryRecordData`]: struct.QueryRecordData.html
pub fn query_record(
	fullname: &str,
	rr_type: Type,
	handle: &Handle,
) -> io::Result<QueryRecord> {
	query_record_extended(fullname, rr_type, QueryRecordData::default(), handle)
}
