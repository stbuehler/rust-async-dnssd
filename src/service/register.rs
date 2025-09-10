use futures_util::StreamExt;
use std::{
	io,
	os::raw::{
		c_char,
		c_void,
	},
	pin::Pin,
	task::{
		Context,
		Poll,
	},
};

use crate::{
	Record,
	cstr,
	dns_consts::Type,
	ffi,
	inner,
	interface::Interface,
};

type CallbackStream = crate::stream::ServiceStream<inner::SharedService, RegisterResult>;

bitflags::bitflags! {
	/// Flags used to register service
	#[derive(Default)]
	pub struct RegisterFlags: ffi::DNSServiceFlags {
		/// Indicates a name conflict should not get handled automatically.
		///
		/// See [`kDNSServiceFlagsNoAutoRename`](https://developer.apple.com/documentation/dnssd/1823436-anonymous/kdnsserviceflagsnoautorename).
		const NO_AUTO_RENAME = ffi::FLAGS_NO_AUTO_RENAME;

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

bitflags::bitflags! {
	/// Flags for [`QueryRecordResult`]
	#[derive(Default)]
	pub struct RegisteredFlags: ffi::DNSServiceFlags {
		/// WARNING: not supported by avahi! Indiciates whether registration was added or removed.
		///
		/// Indicates the registration was succesful.  If not set indicates the
		/// registration was removed.
		///
		/// See [`kDNSServiceFlagsAdd`](https://developer.apple.com/documentation/dnssd/1823436-anonymous/kdnsserviceflagsadd).
		const ADD = ffi::FLAGS_ADD;
	}
}

/// Registration handle
///
/// Can be used to add additional records to the registration.
///
/// Also keeps the registration alive (same as `Registration`).
#[derive(Clone)]
pub struct RegistrationHandle(inner::SharedService);

impl RegistrationHandle {
	/// Add a record to a registered service
	///
	/// See [`DNSServiceAddRecord`](https://developer.apple.com/documentation/dnssd/1804730-dnsserviceaddrecord)
	#[doc(alias = "DNSServiceAddRecord")]
	pub fn add_record(&self, rr_type: Type, rdata: &[u8], ttl: u32) -> io::Result<Record> {
		Ok(self
			.0
			.clone()
			.add_record(0 /* no flags */, rr_type, rdata, ttl)?
			.into())
	}

	/// Get [`Record`] handle for default TXT record
	/// associated with the service registration (e.g. to update it).
	///
	/// [`Record::keep`] doesn't do anything useful on that handle.
	pub fn get_default_txt_record(&self) -> Record {
		self.0.clone().get_default_txt_record().into()
	}
}

/// Registration
///
/// A registration can become active and get deleted again; you need to
/// poll the stream for updates.
///
/// Keeps the registration alive (same as `RegistrationHandle`).
#[must_use = "streams do nothing unless polled"]
pub struct Registration {
	stream: CallbackStream,
	handle: RegistrationHandle,
}

impl std::ops::Deref for Registration {
	type Target = RegistrationHandle;

	fn deref(&self) -> &Self::Target {
		&self.handle
	}
}

impl futures_core::Stream for Registration {
	type Item = io::Result<RegisterResult>;

	fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
		let this = self.get_mut();
		this.stream.poll_next_unpin(cx)
	}
}

/// Service registration result
///
/// See [`DNSServiceRegisterReply`](https://developer.apple.com/documentation/dnssd/dnsserviceregisterreply).
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct RegisterResult {
	/// flags
	pub flags: RegisteredFlags,
	/// if [`NoAutoRename`](enum.RegisterFlag.html#variant.NoAutoRename)
	/// was set this is the original name, otherwise it might be
	/// different.
	pub name: String,
	/// the registered service type
	pub reg_type: String,
	/// domain the service was registered on
	pub domain: String,
}

unsafe extern "C" fn register_callback(
	_sd_ref: ffi::DNSServiceRef,
	flags: ffi::DNSServiceFlags,
	error_code: ffi::DNSServiceErrorType,
	name: *const c_char,
	reg_type: *const c_char,
	domain: *const c_char,
	context: *mut c_void,
) {
	unsafe {
		CallbackStream::run_callback(context, error_code, || {
			let name = cstr::from_cstr(name)?;
			let reg_type = cstr::from_cstr(reg_type)?;
			let domain = cstr::from_cstr(domain)?;

			Ok(RegisterResult {
				flags: RegisteredFlags::from_bits_truncate(flags),
				name: name.to_string(),
				reg_type: reg_type.to_string(),
				domain: domain.to_string(),
			})
		});
	}
}

/// Optional data when registering a service; either use its default
/// value or customize it like:
///
/// ```
/// # use async_dnssd::RegisterData;
/// RegisterData {
///     txt: b"some text data",
///     ..Default::default()
/// };
/// ```
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct RegisterData<'a> {
	/// flags for registration
	pub flags: RegisterFlags,
	/// interface to register service on
	pub interface: Interface,
	/// service name, defaults to hostname
	pub name: Option<&'a str>,
	/// domain on which to advertise the service
	pub domain: Option<&'a str>,
	/// the SRV target host name, defaults to local hostname(s).
	/// Address records are NOT automatically generated for other names.
	pub host: Option<&'a str>,
	/// The TXT record rdata. Empty RDATA is treated like `b"\0"`, i.e.
	/// a TXT record with a single empty string.
	///
	/// You can use [`TxtRecord`][crate::TxtRecord] to create the value for this field
	/// (both [`TxtRecord::data`][crate::TxtRecord::data] and
	/// [`TxtRecord::rdata`][crate::TxtRecord::rdata] produce appropriate values).
	pub txt: &'a [u8],
	#[doc(hidden)]
	pub _non_exhaustive: crate::non_exhaustive_struct::NonExhaustiveMarker,
}

impl<'a> Default for RegisterData<'a> {
	fn default() -> Self {
		Self {
			flags: RegisterFlags::default(),
			interface: Interface::default(),
			name: None,
			domain: None,
			host: None,
			txt: b"",
			_non_exhaustive: crate::non_exhaustive_struct::NonExhaustiveMarker,
		}
	}
}

/// Register a service
///
/// * `reg_type`: the service type followed by the protocol, separated
///   by a dot (for example, "_ssh._tcp").  For details see
///   [`DNSServiceRegister`]
/// * `port`: The port (in native byte order) on which the service
///   accepts connections.  Pass 0 for a "placeholder" service.
/// * `data`: additional service data
///
/// See [`DNSServiceRegister`].
///
/// [`DNSServiceRegister`]: https://developer.apple.com/documentation/dnssd/1804733-dnsserviceregister
#[doc(alias = "DNSServiceRegister")]
#[allow(clippy::too_many_arguments)]
pub fn register_extended(
	reg_type: &str,
	port: u16,
	data: RegisterData<'_>,
) -> io::Result<Registration> {
	crate::init();

	let name = cstr::NullableCStr::from(&data.name)?;
	let reg_type = cstr::CStr::from(&reg_type)?;
	let domain = cstr::NullableCStr::from(&data.domain)?;
	let host = cstr::NullableCStr::from(&data.host)?;

	let stream = CallbackStream::new(move |sender| {
		inner::OwnedService::register(
			data.flags.bits(),
			data.interface.into_raw(),
			&name,
			&reg_type,
			&domain,
			&host,
			port.to_be(),
			data.txt,
			Some(register_callback),
			sender,
		)
		.map(|s| s.share())
	})?;

	let handle = RegistrationHandle(stream.service().clone());
	Ok(Registration { stream, handle })
}

/// Register a service
///
/// * `reg_type`: the service type followed by the protocol, separated
///   by a dot (for example, "_ssh._tcp").  For details see
///   [`DNSServiceRegister`]
/// * `port`: The port (in native byte order) on which the service
///   accepts connections.  Pass 0 for a "placeholder" service.
/// * `handle`: the tokio event loop handle
///
/// Uses [`register_extended`] with default [`RegisterData`].
///
/// See [`DNSServiceRegister`].
///
/// [`DNSServiceRegister`]: https://developer.apple.com/documentation/dnssd/1804733-dnsserviceregister
///
/// # Example
///
/// ```no_run
/// # use async_dnssd::register;
/// # use futures::StreamExt;
/// # #[deny(unused_must_use)]
/// # #[tokio::main(flavor = "current_thread")]
/// # async fn main() -> std::io::Result<()> {
/// let mut registration = async_dnssd::register("_ssh._tcp", 22)?;
/// let result = registration.next().await.expect("no stream end")?;
/// # registration.for_each(|_| async {}).await;
/// # Ok(())
/// # }
/// ```
#[doc(alias = "DNSServiceRegister")]
#[allow(clippy::too_many_arguments)]
pub fn register(reg_type: &str, port: u16) -> io::Result<Registration> {
	register_extended(reg_type, port, RegisterData::default())
}
