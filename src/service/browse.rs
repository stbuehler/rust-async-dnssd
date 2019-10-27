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

use cstr;
use ffi;
use interface::Interface;
use raw;

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
	pub fn resolve(&self) -> io::Result<::Resolve> {
		::resolve(
			self.interface,
			&self.service_name,
			&self.reg_type,
			&self.domain,
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

/// Optional data when browsing for a service; either use its default
/// value or customize it like:
///
/// ```
/// # use async_dnssd::BrowseData;
/// BrowseData {
///     domain: Some("example.com"),
///     ..Default::default()
/// };
/// ```
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct BrowseData<'a> {
	/// interface to query records on
	pub interface: Interface,
	/// domain on which to search for the service
	pub domain: Option<&'a str>,
}

impl<'a> Default for BrowseData<'a> {
	fn default() -> Self {
		BrowseData {
			interface: Interface::default(),
			domain: None,
		}
	}
}

/// Browse for available services
///
/// `reg_type` specifies the service type to search, e.g. `"_ssh._tcp"`.
///
/// See [`DNSServiceBrowse`](https://developer.apple.com/documentation/dnssd/1804742-dnsservicebrowse).
pub fn browse_extended(
	reg_type: &str,
	data: BrowseData,
) -> io::Result<Browse> {
	::init();

	let reg_type = cstr::CStr::from(&reg_type)?;
	let domain = cstr::NullableCStr::from(&data.domain)?;

	Ok(Browse(CallbackStream::new(move |sender| {
		raw::DNSService::browse(
			0, // no flags
			data.interface.into_raw(),
			&reg_type,
			&domain,
			Some(browse_callback),
			sender,
		)
	})?))
}

/// Browse for available services
///
/// `reg_type` specifies the service type to search, e.g. `"_ssh._tcp"`.
///
/// Uses [`browse_extended`] with default [`BrowseData`].
///
/// See [`DNSServiceBrowse`](https://developer.apple.com/documentation/dnssd/1804742-dnsservicebrowse).
///
/// [`browse_extended`]: fn.browse_extended.html
/// [`BrowseData`]: struct.BrowseData.html
pub fn browse(reg_type: &str) -> io::Result<Browse> {
	browse_extended(reg_type, BrowseData::default())
}
