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
use stream::ServiceStream;

/// Set of [`BrowsedFlag`](enum.BrowsedFlag.html)s
///
/// Flags and sets can be combined with bitor (`|`), and bitand (`&`)
/// can be used to test whether a flag is part of a set.
#[derive(Clone,Copy,PartialEq,Eq,PartialOrd,Ord,Hash)]
pub struct BrowsedFlags(u8);

/// Flags for [`BrowseResult`](struct.BrowseResult.html)
#[derive(Clone,Copy,PartialEq,Eq,PartialOrd,Ord,Hash,Debug)]
#[repr(u8)]
pub enum BrowsedFlag {
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

flags_ops!{BrowsedFlags: u8: BrowsedFlag:
	MoreComing,
	Add,
}

flag_mapping!{BrowsedFlags: BrowsedFlag => ffi::DNSServiceFlags:
	MoreComing => ffi::FLAGS_MORE_COMING,
	Add => ffi::FLAGS_ADD,
}

/// Pending browse request
///
/// Results are delivered through `futures::Stream`.
pub struct Browse(ServiceStream<BrowseResult>);

impl futures::Stream for Browse {
	type Item = BrowseResult;
	type Error = io::Error;

	fn poll(&mut self) -> Result<Async<Option<Self::Item>>, Self::Error> {
		self.0.poll()
	}
}

impl GetRemote for Browse {
	fn remote(&self) -> &Remote {
		self.0.remote()
	}
}

/// Browse result
///
/// See [DNSServiceBrowseReply](https://developer.apple.com/documentation/dnssd/dnsservicebrowsereply).
#[derive(Clone,PartialEq,Eq,PartialOrd,Ord,Hash,Debug)]
pub struct BrowseResult{
	/// Flags indicating whether the service was added or removed and
	/// whether there are more pending results.
	pub flags: BrowsedFlags,
	/// Interface the service was found on.
	pub interface: Interface,
	/// Name of the service.
	pub service_name: String,
	/// Type of the service
	pub reg_type: String,
	/// Domain the service was found in
	pub domain: String,
}

impl BrowseResult {
	/// Resolve browse result.
	///
	/// Should check before whether result has the `Add` flag, as
	/// otherwise it probably won't find anything.
	pub fn resolve(&self, handle: &Handle) -> io::Result<::Resolve> {
		::resolve(
			self.interface,
			&self.service_name,
			&self.reg_type,
			&self.domain,
			handle
		)
	}
}

extern "C" fn browse_callback(
	_sd_ref: ffi::DNSServiceRef,
	flags: ffi::DNSServiceFlags,
	interface_index: u32,
	error_code: ffi::DNSServiceErrorType,
	service_name: *const c_char,
	reg_type: *const c_char,
	reply_domain: *const c_char,
	context: *mut c_void
) {
	let sender = context as *mut mpsc::UnboundedSender<io::Result<BrowseResult>>;
	let sender : &mpsc::UnboundedSender<io::Result<BrowseResult>> = unsafe { &*sender };

	let data = Error::from(error_code).map_err(io::Error::from).and_then(|_| {
		let service_name = unsafe { cstr::from_cstr(service_name) }?;
		let reg_type = unsafe { cstr::from_cstr(reg_type) }?;
		let reply_domain = unsafe { cstr::from_cstr(reply_domain) }?;

		Ok(BrowseResult{
			flags: BrowsedFlags::from(flags),
			interface: Interface::from_raw(interface_index),
			service_name: service_name.to_string(),
			reg_type: reg_type.to_string(),
			domain: reply_domain.to_string(),
		})
	});

	sender.unbounded_send(data).unwrap();
}

/// Browse for available services
///
/// `reg_type` specifies the service type to search, e.g. `"_ssh._tcp"`.
///
/// See [`DNSServiceBrowse`](https://developer.apple.com/documentation/dnssd/1804742-dnsservicebrowse).
pub fn browse(
	interface: Interface,
	reg_type: &str,
	domain: Option<&str>,
	handle: &Handle
) -> io::Result<Browse> {
	let reg_type = cstr::CStr::from(&reg_type)?;
	let domain = cstr::NullableCStr::from(&domain)?;

	Ok(Browse(ServiceStream::new(move |sender|
		EventedDNSService::new(
			raw::DNSService::browse(
				0, /* no flags */
				interface.into_raw(),
				&reg_type,
				&domain,
				Some(browse_callback),
				sender as *mut c_void,
			)?,
			handle
		)
	)?))
}
