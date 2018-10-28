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

type CallbackStream = ::stream::ServiceStream<BrowseResult>;

bitflags! {
	/// Flags for [`BrowseResult`](struct.BrowseResult.html)
	#[derive(Default)]
	pub struct BrowsedFlags: ffi::DNSServiceFlags {
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

/// Pending browse request
///
/// Results are delivered through `futures::Stream`.
#[must_use = "streams do nothing unless polled"]
pub struct Browse(CallbackStream);

impl futures::Stream for Browse {
	type Error = io::Error;
	type Item = BrowseResult;

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
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct BrowseResult {
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
			handle,
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
	context: *mut c_void,
) {
	CallbackStream::run_callback(context, error_code, || {
		let service_name = unsafe { cstr::from_cstr(service_name) }?;
		let reg_type = unsafe { cstr::from_cstr(reg_type) }?;
		let reply_domain = unsafe { cstr::from_cstr(reply_domain) }?;

		Ok(BrowseResult {
			flags: BrowsedFlags::from_bits_truncate(flags),
			interface: Interface::from_raw(interface_index),
			service_name: service_name.to_string(),
			reg_type: reg_type.to_string(),
			domain: reply_domain.to_string(),
		})
	});
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
	handle: &Handle,
) -> io::Result<Browse> {
	::init();

	let reg_type = cstr::CStr::from(&reg_type)?;
	let domain = cstr::NullableCStr::from(&domain)?;

	Ok(Browse(CallbackStream::new(handle, move |sender| {
		raw::DNSService::browse(
			0, // no flags
			interface.into_raw(),
			&reg_type,
			&domain,
			Some(browse_callback),
			sender,
		)
	})?))
}
