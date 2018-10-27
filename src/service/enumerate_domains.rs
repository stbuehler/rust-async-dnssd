use futures::{self,Async};
use std::os::raw::{c_void,c_char};
use std::io;
use tokio_core::reactor::{Handle,Remote};

use cstr;
use ffi;
use interface::Interface;
use raw;
use remote::GetRemote;

type CallbackStream = ::stream::ServiceStream<EnumerateResult>;

/// Whether to enumerate domains which are browsed or domains for which
/// registrations can be made.
#[derive(Clone,Copy,PartialEq,Eq,PartialOrd,Ord,Hash,Debug)]
pub enum Enumerate {
	/// enumerate domains which can be browsed
	BrowseDomains,
	/// enumerate domains to register services/records on
	RegistrationDomains,
}

impl Into<ffi::DNSServiceFlags> for Enumerate {
	fn into(self) -> ffi::DNSServiceFlags {
		match self {
			Enumerate::BrowseDomains => ffi::FLAGS_BROWSE_DOMAINS,
			Enumerate::RegistrationDomains => ffi::FLAGS_REGISTRATION_DOMAINS,
		}
	}
}

/// Set of [`EnumeratedFlag`](enum.EnumeratedFlag.html)s
///
/// Flags and sets can be combined with bitor (`|`), and bitand (`&`)
/// can be used to test whether a flag is part of a set.
#[derive(Clone,Copy,PartialEq,Eq,PartialOrd,Ord,Hash)]
pub struct EnumeratedFlags(u8);

/// Flags for [`EnumerateDomains`](struct.EnumerateDomains.html)
#[derive(Clone,Copy,PartialEq,Eq,PartialOrd,Ord,Hash,Debug)]
#[repr(u8)]
pub enum EnumeratedFlag {
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

	/// Indicates this is the default domain to search (always combined with `Add`).
	///
	/// See [`kDNSServiceFlagsDefault`](https://developer.apple.com/documentation/dnssd/1823436-anonymous/kdnsserviceflagsdefault).
	Default,
}

flags_ops!{EnumeratedFlags: u8: EnumeratedFlag:
	MoreComing,
	Add,
	Default,
}

flag_mapping!{EnumeratedFlags: EnumeratedFlag => ffi::DNSServiceFlags:
	MoreComing => ffi::FLAGS_MORE_COMING,
	Add => ffi::FLAGS_ADD,
	Default => ffi::FLAGS_DEFAULT,
}

/// Pending domain enumeration
pub struct EnumerateDomains(CallbackStream);

impl futures::Stream for EnumerateDomains {
	type Item = EnumerateResult;
	type Error = io::Error;

	fn poll(&mut self) -> Result<Async<Option<Self::Item>>, Self::Error> {
		self.0.poll()
	}
}

impl GetRemote for EnumerateDomains {
	fn remote(&self) -> &Remote {
		self.0.remote()
	}
}

/// Domain enumeration result
///
/// See [DNSServiceDomainEnumReply](https://developer.apple.com/documentation/dnssd/dnsservicedomainenumreply).
#[derive(Clone,PartialEq,Eq,PartialOrd,Ord,Hash,Debug)]
pub struct EnumerateResult{
	///
	pub flags: EnumeratedFlags,
	///
	pub interface: Interface,
	///
	pub domain: String,
}

extern "C" fn enumerate_callback(
	_sd_ref: ffi::DNSServiceRef,
	flags: ffi::DNSServiceFlags,
	interface_index: u32,
	error_code: ffi::DNSServiceErrorType,
	reply_domain: *const c_char,
	context: *mut c_void
) {
	CallbackStream::run_callback(context, error_code, || {
		let reply_domain = unsafe { cstr::from_cstr(reply_domain) }?;

		Ok(EnumerateResult{
			flags: EnumeratedFlags::from(flags),
			interface: Interface::from_raw(interface_index),
			domain: reply_domain.to_string(),
		})
	});
}

/// Enumerate domains that are recommended for registration or browsing
///
/// See [`DNSServiceEnumerateDomains`](https://developer.apple.com/documentation/dnssd/1804754-dnsserviceenumeratedomains).
pub fn enumerate_domains(enumerate: Enumerate, interface: Interface, handle: &Handle) -> io::Result<EnumerateDomains> {
	::init();

	Ok(EnumerateDomains(CallbackStream::new(handle, move |sender|
		raw::DNSService::enumerate_domains(
			enumerate.into(),
			interface.into_raw(),
			Some(enumerate_callback),
			sender,
		)
	)?))
}
