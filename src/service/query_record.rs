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
use ffi;
use interface::Interface;
use raw;
use remote::GetRemote;

type CallbackStream = ::stream::ServiceStream<QueryRecordResult>;

/// Set of [`QueryRecordFlag`](enum.QueryRecordFlag.html)s
///
/// Flags and sets can be combined with bitor (`|`), and bitand (`&`)
/// can be used to test whether a flag is part of a set.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct QueryRecordFlags(u8);

/// Flags used to query for a record
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(u8)]
pub enum QueryRecordFlag {
	/// long-lived unicast query
	///
	/// See [`kDNSServiceFlagsLongLivedQuery`](https://developer.apple.com/documentation/dnssd/1823436-anonymous/kdnsserviceflagslonglivedquery).
	LongLivedQuery = 0,
}

flags_ops!{QueryRecordFlags: u8: QueryRecordFlag:
	LongLivedQuery,
}

flag_mapping!{QueryRecordFlags: QueryRecordFlag => ffi::DNSServiceFlags:
	LongLivedQuery => ffi::FLAGS_LONG_LIVED_QUERY,
}

/// Set of [`QueriedRecordFlag`](enum.QueriedRecordFlag.html)s
///
/// Flags and sets can be combined with bitor (`|`), and bitand (`&`)
/// can be used to test whether a flag is part of a set.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct QueriedRecordFlags(u8);

/// Flags for [`QueryRecordResult`](struct.QueryRecordResult.html)
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(u8)]
pub enum QueriedRecordFlag {
	/// Indicates at least one more result is pending in the queue.  If
	/// not set there still might be more results coming in the future.
	///
	/// See [`kDNSServiceFlagsMoreComing`](https://developer.apple.com/documentation/dnssd/1823436-anonymous/kdnsserviceflagsmorecoming).
	MoreComing = 0,

	/// Indicates the result is new.  If not set indicates the result
	/// was removed.
	///
	/// See [`kDNSServiceFlagsAdd`](https://developer.apple.com/documentation/dnssd/1823436-anonymous/kdnsserviceflagsadd).
	Add,
}

flags_ops!{QueriedRecordFlags: u8: QueriedRecordFlag:
	MoreComing,
	Add,
}

flag_mapping!{QueriedRecordFlags: QueriedRecordFlag => ffi::DNSServiceFlags:
	MoreComing => ffi::FLAGS_MORE_COMING,
	Add => ffi::FLAGS_ADD,
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
	///
	pub flags: QueriedRecordFlags,
	///
	pub interface: Interface,
	///
	pub fullname: String,
	///
	pub rr_type: u16,
	///
	pub rr_class: u16,
	///
	pub rdata: Vec<u8>,
	///
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
			flags: QueriedRecordFlags::from(flags),
			interface: Interface::from_raw(interface_index),
			fullname: fullname.to_string(),
			rr_type,
			rr_class,
			rdata: rdata.into(),
			ttl,
		})
	});
}

/// Query for an arbitrary DNS record
///
/// See [`DNSServiceQueryRecord`](https://developer.apple.com/documentation/dnssd/1804747-dnsservicequeryrecordc).
pub fn query_record(
	flags: QueryRecordFlags,
	interface: Interface,
	fullname: &str,
	rr_type: u16,
	rr_class: u16,
	handle: &Handle,
) -> io::Result<QueryRecord> {
	::init();

	let fullname = cstr::CStr::from(&fullname)?;

	Ok(QueryRecord(CallbackStream::new(handle, move |sender| {
		raw::DNSService::query_record(
			flags.into(),
			interface.into_raw(),
			&fullname,
			rr_type,
			rr_class,
			Some(query_record_callback),
			sender,
		)
	})?))
}
