#![allow(clippy::too_many_arguments)]

use futures::{
	lock,
	prelude::*,
};
use libc::c_void;
use std::{
	io,
	pin::Pin,
	ptr::null_mut,
	sync::{
		Arc,
		Mutex,
	},
	task::{
		Context,
		Poll,
	},
};

use crate::{
	cstr,
	dns_consts::{
		Class,
		Type,
	},
	error::Error,
	ffi,
};

// More typesafe than raw "ffi", but still not quite done

struct ManagedService(ffi::DNSServiceRef);

unsafe impl Send for ManagedService {}
unsafe impl Sync for ManagedService {}

impl Drop for ManagedService {
	fn drop(&mut self) {
		unsafe {
			ffi::DNSServiceRefDeallocate(self.0);
		}
	}
}

/// Keeps the service alive
#[derive(Clone)]
pub struct ServiceHandle {
	managed_service: Arc<ManagedService>,
}

impl ServiceHandle {
	fn new(raw: ffi::DNSServiceRef) -> Self {
		Self {
			managed_service: Arc::new(ManagedService(raw)),
		}
	}

	fn as_raw(&self) -> ffi::DNSServiceRef {
		self.managed_service.0
	}
}

pub(crate) trait EventedService: Unpin {
	fn poll_service(&mut self, cx: &mut Context<'_>) -> io::Result<()>;
}

/// Many places can keep the service alive, but a single active user
pub(crate) struct OwnedService {
	handle: ServiceHandle,
	processing: crate::evented::ReadProcessor,
}

impl OwnedService {
	fn new(raw: ffi::DNSServiceRef) -> io::Result<Self> {
		let fd = unsafe { ffi::DNSServiceRefSockFD(raw) };
		let handle = ServiceHandle::new(raw);
		let processing = crate::evented::ReadProcessor::new(fd)?;
		Ok(Self { handle, processing })
	}

	pub(crate) fn share(self) -> SharedService {
		SharedService {
			handle: Arc::new(Mutex::new(self.handle.clone())),
			service: Arc::new(lock::Mutex::new(self)),
			lock_state: SharedLockState::Clear,
		}
	}

	// -----

	pub(crate) fn enumerate_domains(
		flags: ffi::DNSServiceFlags,
		interface_index: u32,
		callback: ffi::DNSServiceDomainEnumReply,
		context: *mut c_void,
	) -> Result<Self, Error> {
		let mut sd_ref: ffi::DNSServiceRef = null_mut();
		Error::from(unsafe {
			ffi::DNSServiceEnumerateDomains(&mut sd_ref, flags, interface_index, callback, context)
		})?;
		Ok(Self::new(sd_ref)?)
	}

	pub(crate) fn register(
		flags: ffi::DNSServiceFlags,
		interface_index: u32,
		name: &cstr::NullableCStr<'_>,
		reg_type: &cstr::CStr<'_>,
		domain: &cstr::NullableCStr<'_>,
		host: &cstr::NullableCStr<'_>,
		port: u16,
		txt: &[u8],
		callback: ffi::DNSServiceRegisterReply,
		context: *mut c_void,
	) -> Result<Self, Error> {
		let txt_len = txt.len();
		assert!(txt_len < (1 << 16));
		let txt_len = txt_len as u16;
		let txt_record = txt.as_ptr();

		let mut sd_ref: ffi::DNSServiceRef = null_mut();
		Error::from(unsafe {
			ffi::DNSServiceRegister(
				&mut sd_ref,
				flags,
				interface_index,
				name.as_ptr(),
				reg_type.as_ptr(),
				domain.as_ptr(),
				host.as_ptr(),
				port,
				txt_len,
				txt_record,
				callback,
				context,
			)
		})?;
		Ok(Self::new(sd_ref)?)
	}

	pub(crate) fn browse(
		flags: ffi::DNSServiceFlags,
		interface_index: u32,
		reg_type: &cstr::CStr<'_>,
		domain: &cstr::NullableCStr<'_>,
		callback: ffi::DNSServiceBrowseReply,
		context: *mut c_void,
	) -> Result<Self, Error> {
		let mut sd_ref: ffi::DNSServiceRef = null_mut();
		Error::from(unsafe {
			ffi::DNSServiceBrowse(
				&mut sd_ref,
				flags,
				interface_index,
				reg_type.as_ptr(),
				domain.as_ptr(),
				callback,
				context,
			)
		})?;
		Ok(Self::new(sd_ref)?)
	}

	pub(crate) fn resolve(
		flags: ffi::DNSServiceFlags,
		interface_index: u32,
		name: &cstr::CStr<'_>,
		reg_type: &cstr::CStr<'_>,
		domain: &cstr::CStr<'_>,
		callback: ffi::DNSServiceResolveReply,
		context: *mut c_void,
	) -> Result<Self, Error> {
		let mut sd_ref: ffi::DNSServiceRef = null_mut();
		Error::from(unsafe {
			ffi::DNSServiceResolve(
				&mut sd_ref,
				flags,
				interface_index,
				name.as_ptr(),
				reg_type.as_ptr(),
				domain.as_ptr(),
				callback,
				context,
			)
		})?;
		Ok(Self::new(sd_ref)?)
	}

	pub(crate) fn query_record(
		flags: ffi::DNSServiceFlags,
		interface_index: u32,
		fullname: &cstr::CStr<'_>,
		rr_type: Type,
		rr_class: Class,
		callback: ffi::DNSServiceQueryRecordReply,
		context: *mut c_void,
	) -> Result<Self, Error> {
		let mut sd_ref: ffi::DNSServiceRef = null_mut();
		Error::from(unsafe {
			ffi::DNSServiceQueryRecord(
				&mut sd_ref,
				flags,
				interface_index,
				fullname.as_ptr(),
				rr_type.0,
				rr_class.0,
				callback,
				context,
			)
		})?;
		Ok(Self::new(sd_ref)?)
	}
}

impl EventedService for OwnedService {
	fn poll_service(&mut self, cx: &mut Context<'_>) -> io::Result<()> {
		let raw = self.handle.as_raw();
		self.processing.process(cx, || {
			Error::from(unsafe { ffi::DNSServiceProcessResult(raw) })?;
			Ok(())
		})
	}
}

enum SharedLockState {
	Clear,
	Wait(lock::MutexLockFuture<'static, OwnedService>),
	Locked(lock::MutexGuard<'static, OwnedService>),
}

pub(crate) struct SharedService {
	handle: Arc<Mutex<ServiceHandle>>,       // protecting ffi calls
	service: Arc<lock::Mutex<OwnedService>>, // coordinate which "task" polls
	lock_state: SharedLockState,
}

impl EventedService for SharedService {
	fn poll_service(&mut self, cx: &mut Context<'_>) -> io::Result<()> {
		if let SharedLockState::Clear = self.lock_state {
			let lf: lock::MutexLockFuture<'_, OwnedService> = self.service.lock();
			// we'll clear the lock state before releasing the mutex
			let lf: lock::MutexLockFuture<'static, OwnedService> =
				unsafe { std::mem::transmute(lf) };
			self.lock_state = SharedLockState::Wait(lf);
		}
		if let SharedLockState::Wait(wait) = &mut self.lock_state {
			match Pin::new(wait).poll(cx) {
				Poll::Pending => return Ok(()),
				Poll::Ready(g) => {
					self.lock_state = SharedLockState::Locked(g);
				},
			}
		}
		if let SharedLockState::Locked(guard) = &mut self.lock_state {
			let _guard = self.handle.lock().unwrap();
			guard.poll_service(cx)?;
		}

		Ok(())
	}
}

impl Clone for SharedService {
	fn clone(&self) -> Self {
		Self {
			handle: self.handle.clone(),
			service: self.service.clone(),
			lock_state: SharedLockState::Clear,
		}
	}
}

impl SharedService {
	pub(crate) fn get_default_txt_record(self) -> DNSRecord {
		DNSRecord {
			service: self,
			raw: DNSRecordRef(null_mut()),
			rr_type: Type::TXT,
		}
	}

	// only valid when `service` was created through "register"
	//
	// still needs a SharedService to synchronize ffi calls
	pub(crate) fn add_record(
		self,
		flags: ffi::DNSServiceFlags,
		rr_type: Type,
		rdata: &[u8],
		ttl: u32,
	) -> Result<DNSRecord, Error> {
		let rd_len = rdata.len();
		assert!(rd_len < (1 << 16));
		let rd_len = rd_len as u16;
		let rdata = rdata.as_ptr();

		let handle = self.handle.lock().unwrap();

		let mut record_ref: ffi::DNSRecordRef = null_mut();
		Error::from(unsafe {
			ffi::DNSServiceAddRecord(
				handle.as_raw(),
				&mut record_ref,
				flags,
				rr_type.0,
				rd_len,
				rdata,
				ttl,
			)
		})?;

		drop(handle);

		Ok(DNSRecord {
			service: self,
			raw: DNSRecordRef(record_ref),
			rr_type,
		})
	}

	pub(crate) fn create_connection() -> Result<Self, Error> {
		let mut sd_ref: ffi::DNSServiceRef = null_mut();
		Error::from(unsafe { ffi::DNSServiceCreateConnection(&mut sd_ref) })?;
		Ok(OwnedService::new(sd_ref)?.share())
	}

	// only valid when `service` was created through "create_connection"
	pub(crate) fn register_record(
		self,
		flags: ffi::DNSServiceFlags,
		interface_index: u32,
		fullname: &cstr::CStr<'_>,
		rr_type: Type,
		rr_class: Class,
		rdata: &[u8],
		ttl: u32,
		callback: ffi::DNSServiceRegisterRecordReply,
		context: *mut c_void,
	) -> Result<DNSRecord, Error> {
		let rd_len = rdata.len();
		assert!(rd_len < (1 << 16));
		let rd_len = rd_len as u16;
		let rdata = rdata.as_ptr();

		let handle = self.handle.lock().unwrap();

		let mut record_ref: ffi::DNSRecordRef = null_mut();
		Error::from(unsafe {
			ffi::DNSServiceRegisterRecord(
				handle.as_raw(),
				&mut record_ref,
				flags,
				interface_index,
				fullname.as_ptr(),
				rr_type.0,
				rr_class.0,
				rd_len,
				rdata,
				ttl,
				callback,
				context,
			)
		})?;

		drop(handle);

		Ok(DNSRecord {
			service: self,
			raw: DNSRecordRef(record_ref),
			rr_type,
		})
	}
}

impl Drop for SharedService {
	fn drop(&mut self) {
		// first release lock before potentially freeing the mutex
		self.lock_state = SharedLockState::Clear;
	}
}

// so we don't have to unsafe impl for whole `DNSRecord`
//
// can only be used in combination with service handle, which is protected by mutex
struct DNSRecordRef(ffi::DNSRecordRef);

unsafe impl Sync for DNSRecordRef {}
unsafe impl Send for DNSRecordRef {}

pub(crate) struct DNSRecord {
	service: SharedService,
	raw: DNSRecordRef,
	rr_type: Type,
}

impl Drop for DNSRecord {
	fn drop(&mut self) {
		if !self.raw.0.is_null() {
			let handle = self.service.handle.lock().unwrap();
			unsafe {
				ffi::DNSServiceRemoveRecord(
					handle.as_raw(),
					self.raw.0,
					0, // no flags
				);
			}
		}
	}
}

impl DNSRecord {
	pub(crate) fn update_record(
		&self,
		flags: ffi::DNSServiceFlags,
		rdata: &[u8],
		ttl: u32,
	) -> Result<(), Error> {
		let rd_len = rdata.len();
		assert!(rd_len < (1 << 16));
		let rd_len = rd_len as u16;
		let rdata = rdata.as_ptr();

		let handle = self.service.handle.lock().unwrap();

		Error::from(unsafe {
			ffi::DNSServiceUpdateRecord(handle.as_raw(), self.raw.0, flags, rd_len, rdata, ttl)
		})
	}

	pub(crate) fn rr_type(&self) -> Type {
		self.rr_type
	}

	// keep "forever" (until service is dropped)
	pub(crate) fn keep(mut self) {
		self.raw.0 = null_mut();
	}
}

pub fn reconfirm_record(
	flags: ffi::DNSServiceFlags,
	interface_index: u32,
	fullname: &cstr::CStr<'_>,
	rr_type: Type,
	rr_class: Class,
	rdata: &[u8],
) {
	let rd_len = rdata.len();
	assert!(rd_len < (1 << 16));
	let rd_len = rd_len as u16;
	let rdata = rdata.as_ptr();

	unsafe {
		ffi::DNSServiceReconfirmRecord(
			flags,
			interface_index,
			fullname.as_ptr(),
			rr_type.0,
			rr_class.0,
			rd_len,
			rdata,
		);
	}
}
