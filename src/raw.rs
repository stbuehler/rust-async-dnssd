use std::os::raw::{c_int,c_void};
use std::cell::UnsafeCell;
use std::ptr::null_mut;
use std::rc::Rc;

use cstr;
use error::Error;
use ffi;

type FFIResult<R> = Result<R, Error>;

struct InnerDNSService(ffi::DNSServiceRef);

impl Drop for InnerDNSService {
	fn drop(&mut self) {
		unsafe {
			ffi::DNSServiceRefDeallocate(self.0);
		}
	}
}

impl InnerDNSService {
	fn fd(&mut self) -> c_int {
		unsafe { ffi::DNSServiceRefSockFD(self.0) }
	}

	fn process_result(&mut self) -> FFIResult<()> {
		Error::from(unsafe {
			ffi::DNSServiceProcessResult(self.0)
		})
	}

	fn enumerate_domains(
		flags: ffi::DNSServiceFlags,
		interface_index: u32,
		callback: ffi::DNSServiceDomainEnumReply,
		context: *mut c_void
	) -> FFIResult<InnerDNSService> {
		let mut sd_ref : ffi::DNSServiceRef = null_mut();
		Error::from(unsafe {
			ffi::DNSServiceEnumerateDomains(&mut sd_ref, flags, interface_index, callback, context)
		})?;
		Ok(InnerDNSService(sd_ref))
	}

	fn register(
		flags: ffi::DNSServiceFlags,
		interface_index: u32,
		name: &cstr::NullableCStr,
		reg_type: &cstr::CStr,
		domain: &cstr::NullableCStr,
		host: &cstr::NullableCStr,
		port: u16,
		txt: &[u8],
		callback: ffi::DNSServiceRegisterReply,
		context: *mut c_void
	) -> FFIResult<InnerDNSService> {
		let txt_len = txt.len();
		assert!(txt_len < (1 << 16));
		let txt_len = txt_len as u16;
		let txt_record = txt.as_ptr();

		let mut sd_ref : ffi::DNSServiceRef = null_mut();
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
				context
			)
		})?;
		Ok(InnerDNSService(sd_ref))
	}

	fn browse(
		flags: ffi::DNSServiceFlags,
		interface_index: u32,
		reg_type: &cstr::CStr,
		domain: &cstr::NullableCStr,
		callback: ffi::DNSServiceBrowseReply,
		context: *mut c_void
	) -> FFIResult<InnerDNSService> {
		let mut sd_ref : ffi::DNSServiceRef = null_mut();
		Error::from(unsafe {
			ffi::DNSServiceBrowse(
				&mut sd_ref,
				flags,
				interface_index,
				reg_type.as_ptr(),
				domain.as_ptr(),
				callback,
				context
			)
		})?;
		Ok(InnerDNSService(sd_ref))
	}

	fn resolve(
		flags: ffi::DNSServiceFlags,
		interface_index: u32,
		name: &cstr::CStr,
		reg_type: &cstr::CStr,
		domain: &cstr::CStr,
		callback: ffi::DNSServiceResolveReply,
		context: *mut c_void
	) -> FFIResult<InnerDNSService> {
		let mut sd_ref : ffi::DNSServiceRef = null_mut();
		Error::from(unsafe {
			ffi::DNSServiceResolve(
				&mut sd_ref,
				flags,
				interface_index,
				name.as_ptr(),
				reg_type.as_ptr(),
				domain.as_ptr(),
				callback,
				context
			)
		})?;
		Ok(InnerDNSService(sd_ref))
	}

	fn create_connection() -> FFIResult<InnerDNSService> {
		let mut sd_ref : ffi::DNSServiceRef = null_mut();
		Error::from(unsafe {
			ffi::DNSServiceCreateConnection(&mut sd_ref)
		})?;
		Ok(InnerDNSService(sd_ref))
	}

	fn query_record(
		flags: ffi::DNSServiceFlags,
		interface_index: u32,
		fullname: &cstr::CStr,
		rr_type: u16,
		rr_class: u16,
		callback: ffi::DNSServiceQueryRecordReply,
		context: *mut c_void
	) -> FFIResult<InnerDNSService> {
		let mut sd_ref : ffi::DNSServiceRef = null_mut();
		Error::from(unsafe {
			ffi::DNSServiceQueryRecord(
				&mut sd_ref,
				flags,
				interface_index,
				fullname.as_ptr(),
				rr_type,
				rr_class,
				callback,
				context
			)
		})?;
		Ok(InnerDNSService(sd_ref))
	}
}

#[derive(Clone)]
pub struct DNSService(Rc<UnsafeCell<InnerDNSService>>);

impl DNSService {
	fn get(&self) -> &mut InnerDNSService {
		// Rc means it cannot be shared across threads.
		// we need to make sure our callbacks don't ever come back
		// here, otherwise there is no way for "re-entrance".
		let r = self.0.get();
		unsafe { &mut *r }
	}

	fn new(s: FFIResult<InnerDNSService>) -> FFIResult<DNSService> {
		s.map(|s| DNSService(Rc::new(UnsafeCell::new(s))))
	}

	pub fn fd(&self) -> c_int {
		self.get().fd()
	}

	pub fn process_result(&self) -> FFIResult<()> {
		self.get().process_result()
	}

	pub fn enumerate_domains(
		flags: ffi::DNSServiceFlags,
		interface_index: u32,
		callback: ffi::DNSServiceDomainEnumReply,
		context: *mut c_void
	) -> FFIResult<DNSService> {
		Self::new(
			InnerDNSService::enumerate_domains(flags, interface_index, callback, context)
		)
	}

	pub fn register(
		flags: ffi::DNSServiceFlags,
		interface_index: u32,
		name: &cstr::NullableCStr,
		reg_type: &cstr::CStr,
		domain: &cstr::NullableCStr,
		host: &cstr::NullableCStr,
		port: u16,
		txt: &[u8],
		callback: ffi::DNSServiceRegisterReply,
		context: *mut c_void
	) -> FFIResult<DNSService> {
		Self::new(
			InnerDNSService::register(flags, interface_index, name, reg_type, domain, host, port, txt, callback, context)
		)
	}

	pub fn add_record(
		&self,
		flags: ffi::DNSServiceFlags,
		rr_type: u16,
		rdata: &[u8],
		ttl: u32
	) -> FFIResult<DNSRecord> {
		Ok(DNSRecord(
			InnerDNSRecord::add_record(self, flags, rr_type, rdata, ttl)?
		))
	}

	pub fn get_default_txt_record(&self) -> DNSRecord {
		const RR_TYPE_TXT : u16 = 16;
		DNSRecord(
			InnerDNSRecord(self.clone(), null_mut(), RR_TYPE_TXT)
		)
	}

	pub fn browse(
		flags: ffi::DNSServiceFlags,
		interface_index: u32,
		reg_type: &cstr::CStr,
		domain: &cstr::NullableCStr,
		callback: ffi::DNSServiceBrowseReply,
		context: *mut c_void
	) -> FFIResult<DNSService> {
		Self::new(
			InnerDNSService::browse(flags, interface_index, reg_type, domain, callback, context)
		)
	}

	pub fn resolve(
		flags: ffi::DNSServiceFlags,
		interface_index: u32,
		name: &cstr::CStr,
		reg_type: &cstr::CStr,
		domain: &cstr::CStr,
		callback: ffi::DNSServiceResolveReply,
		context: *mut c_void
	) -> FFIResult<DNSService> {
		Self::new(
			InnerDNSService::resolve(flags, interface_index, name, reg_type, domain, callback, context)
		)
	}

	pub fn register_record(&self,
		flags: ffi::DNSServiceFlags,
		interface_index: u32,
		fullname: &cstr::CStr,
		rr_type: u16,
		rr_class: u16,
		rdata: &[u8],
		ttl: u32,
		callback: ffi::DNSServiceRegisterRecordReply,
		context: *mut c_void
	) -> FFIResult<DNSRecord> {
		Ok(DNSRecord(
			InnerDNSRecord::register_record(
				self,
				flags,
				interface_index,
				fullname,
				rr_type,
				rr_class,
				rdata,
				ttl,
				callback,
				context
			)?
		))
	}

	pub fn create_connection() -> FFIResult<DNSService> {
		Self::new(
			InnerDNSService::create_connection()
		)
	}

	pub fn query_record(
		flags: ffi::DNSServiceFlags,
		interface_index: u32,
		fullname: &cstr::CStr,
		rr_type: u16,
		rr_class: u16,
		callback: ffi::DNSServiceQueryRecordReply,
		context: *mut c_void
	) -> FFIResult<DNSService> {
		Self::new(
			InnerDNSService::query_record(flags, interface_index, fullname, rr_type, rr_class, callback, context)
		)
	}
}

struct InnerDNSRecord(DNSService, ffi::DNSRecordRef, u16);

impl Drop for InnerDNSRecord {
	fn drop(&mut self) {
		if !self.1.is_null() {
			unsafe {
				ffi::DNSServiceRemoveRecord(
					self.get_service().0,
					self.1,
					0 /* no flags */
				);
			}
		}
	}
}

impl InnerDNSRecord {
	fn get_service(&self) -> &mut InnerDNSService {
		self.0.get()
	}

	fn rr_type(&self) -> u16 {
		self.2
	}

	// only valid when `service` was created through "register"
	fn add_record(
		service: &DNSService,
		flags: ffi::DNSServiceFlags,
		rr_type: u16,
		rdata: &[u8],
		ttl: u32
	) -> FFIResult<InnerDNSRecord> {
		let rd_len = rdata.len();
		assert!(rd_len < (1 << 16));
		let rd_len = rd_len as u16;
		let rdata = rdata.as_ptr();

		let mut record_ref: ffi::DNSRecordRef = null_mut();
		Error::from(unsafe {
			ffi::DNSServiceAddRecord(
				service.get().0,
				&mut record_ref,
				flags,
				rr_type,
				rd_len,
				rdata,
				ttl
			)
		})?;
		Ok(InnerDNSRecord(service.clone(), record_ref, rr_type))
	}

	// only valid when `service` was created through "create_connection"
	fn register_record(
		service: &DNSService,
		flags: ffi::DNSServiceFlags,
		interface_index: u32,
		fullname: &cstr::CStr,
		rr_type: u16,
		rr_class: u16,
		rdata: &[u8],
		ttl: u32,
		callback: ffi::DNSServiceRegisterRecordReply,
		context: *mut c_void
	) -> FFIResult<InnerDNSRecord> {
		let rd_len = rdata.len();
		assert!(rd_len < (1 << 16));
		let rd_len = rd_len as u16;
		let rdata = rdata.as_ptr();

		let mut record_ref: ffi::DNSRecordRef = null_mut();
		Error::from(unsafe {
			ffi::DNSServiceRegisterRecord(
				service.get().0,
				&mut record_ref,
				flags,
				interface_index,
				fullname.as_ptr(),
				rr_type,
				rr_class,
				rd_len,
				rdata,
				ttl,
				callback,
				context
			)
		})?;
		Ok(InnerDNSRecord(service.clone(), record_ref, rr_type))
	}

	fn update_record(
		&self,
		flags: ffi::DNSServiceFlags,
		rdata: &[u8],
		ttl: u32
	) -> FFIResult<()> {
		let rd_len = rdata.len();
		assert!(rd_len < (1 << 16));
		let rd_len = rd_len as u16;
		let rdata = rdata.as_ptr();

		Error::from(unsafe {
			ffi::DNSServiceUpdateRecord(
				self.get_service().0,
				self.1,
				flags,
				rd_len,
				rdata,
				ttl
			)
		})
	}

	fn keep(mut self) {
		self.1 = null_mut();
	}
}

pub struct DNSRecord(InnerDNSRecord);

impl DNSRecord {
	pub fn rr_type(&self) -> u16 {
		self.0.rr_type()
	}

	pub fn update_record(
		&self,
		flags: ffi::DNSServiceFlags,
		rdata: &[u8],
		ttl: u32
	) -> FFIResult<()> {
		self.0.update_record(flags, rdata, ttl)
	}

	// keep "forever" (until service is dropped)
	pub fn keep(self) {
		self.0.keep()
	}
}

pub fn reconfirm_record(
	flags: ffi::DNSServiceFlags,
	interface_index: u32,
	fullname: &cstr::CStr,
	rr_type: u16,
	rr_class: u16,
	rdata: &[u8]
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
			rr_type,
			rr_class,
			rd_len,
			rdata
		);
	}
}
