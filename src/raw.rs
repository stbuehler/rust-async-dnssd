#![allow(clippy::too_many_arguments)]

use std::{
	cell::UnsafeCell,
	os::raw::{
		c_int,
		c_void,
	},
	ptr::null_mut,
	rc::Rc,
};

use crate::cstr;
use crate::dns_consts::{
	Class,
	Type,
};
use crate::error::Error;
use crate::ffi;

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
	fn fd(&self) -> c_int {
		unsafe { ffi::DNSServiceRefSockFD(self.0) }
	}

	fn process_result(&self) -> FFIResult<()> {
		Error::from(unsafe { ffi::DNSServiceProcessResult(self.0) })
	}

	fn enumerate_domains(
		flags: ffi::DNSServiceFlags,
		interface_index: u32,
		callback: ffi::DNSServiceDomainEnumReply,
		context: *mut c_void,
	) -> FFIResult<InnerDNSService> {
		let mut sd_ref: ffi::DNSServiceRef = null_mut();
		Error::from(unsafe {
			ffi::DNSServiceEnumerateDomains(
				&mut sd_ref,
				flags,
				interface_index,
				callback,
				context,
			)
		})?;
		Ok(InnerDNSService(sd_ref))
	}

	fn register(
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
	) -> FFIResult<InnerDNSService> {
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
		Ok(InnerDNSService(sd_ref))
	}

	fn browse(
		flags: ffi::DNSServiceFlags,
		interface_index: u32,
		reg_type: &cstr::CStr<'_>,
		domain: &cstr::NullableCStr<'_>,
		callback: ffi::DNSServiceBrowseReply,
		context: *mut c_void,
	) -> FFIResult<InnerDNSService> {
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
		Ok(InnerDNSService(sd_ref))
	}

	fn resolve(
		flags: ffi::DNSServiceFlags,
		interface_index: u32,
		name: &cstr::CStr<'_>,
		reg_type: &cstr::CStr<'_>,
		domain: &cstr::CStr<'_>,
		callback: ffi::DNSServiceResolveReply,
		context: *mut c_void,
	) -> FFIResult<InnerDNSService> {
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
		Ok(InnerDNSService(sd_ref))
	}

	fn create_connection() -> FFIResult<InnerDNSService> {
		let mut sd_ref: ffi::DNSServiceRef = null_mut();
		Error::from(unsafe { ffi::DNSServiceCreateConnection(&mut sd_ref) })?;
		Ok(InnerDNSService(sd_ref))
	}

	fn query_record(
		flags: ffi::DNSServiceFlags,
		interface_index: u32,
		fullname: &cstr::CStr<'_>,
		rr_type: Type,
		rr_class: Class,
		callback: ffi::DNSServiceQueryRecordReply,
		context: *mut c_void,
	) -> FFIResult<InnerDNSService> {
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
		Ok(InnerDNSService(sd_ref))
	}
}

#[derive(Clone)]
pub(crate) struct DNSService(Rc<UnsafeCell<InnerDNSService>>);

impl DNSService {
	fn get(&self) -> &InnerDNSService {
		let r = self.0.get();
		unsafe { &*r }
	}

	fn new(s: FFIResult<InnerDNSService>) -> FFIResult<DNSService> {
		s.map(|s| DNSService(Rc::new(UnsafeCell::new(s))))
	}

	pub(crate) fn fd(&self) -> c_int {
		self.get().fd()
	}

	pub(crate) fn process_result(&self) -> FFIResult<()> {
		self.get().process_result()
	}

	pub(crate) fn enumerate_domains(
		flags: ffi::DNSServiceFlags,
		interface_index: u32,
		callback: ffi::DNSServiceDomainEnumReply,
		context: *mut c_void,
	) -> FFIResult<DNSService> {
		Self::new(InnerDNSService::enumerate_domains(
			flags,
			interface_index,
			callback,
			context,
		))
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
	) -> FFIResult<DNSService> {
		Self::new(InnerDNSService::register(
			flags,
			interface_index,
			name,
			reg_type,
			domain,
			host,
			port,
			txt,
			callback,
			context,
		))
	}

	pub(crate) fn add_record(
		&self,
		flags: ffi::DNSServiceFlags,
		rr_type: Type,
		rdata: &[u8],
		ttl: u32,
	) -> FFIResult<DNSRecord> {
		Ok(DNSRecord(InnerDNSRecord::add_record(
			self, flags, rr_type, rdata, ttl,
		)?))
	}

	pub(crate) fn get_default_txt_record(&self) -> DNSRecord {
		DNSRecord(InnerDNSRecord(self.clone(), null_mut(), Type::TXT))
	}

	pub(crate) fn browse(
		flags: ffi::DNSServiceFlags,
		interface_index: u32,
		reg_type: &cstr::CStr<'_>,
		domain: &cstr::NullableCStr<'_>,
		callback: ffi::DNSServiceBrowseReply,
		context: *mut c_void,
	) -> FFIResult<DNSService> {
		Self::new(InnerDNSService::browse(
			flags,
			interface_index,
			reg_type,
			domain,
			callback,
			context,
		))
	}

	pub(crate) fn resolve(
		flags: ffi::DNSServiceFlags,
		interface_index: u32,
		name: &cstr::CStr<'_>,
		reg_type: &cstr::CStr<'_>,
		domain: &cstr::CStr<'_>,
		callback: ffi::DNSServiceResolveReply,
		context: *mut c_void,
	) -> FFIResult<DNSService> {
		Self::new(InnerDNSService::resolve(
			flags,
			interface_index,
			name,
			reg_type,
			domain,
			callback,
			context,
		))
	}

	pub(crate) fn register_record(
		&self,
		flags: ffi::DNSServiceFlags,
		interface_index: u32,
		fullname: &cstr::CStr<'_>,
		rr_type: Type,
		rr_class: Class,
		rdata: &[u8],
		ttl: u32,
		callback: ffi::DNSServiceRegisterRecordReply,
		context: *mut c_void,
	) -> FFIResult<DNSRecord> {
		Ok(DNSRecord(InnerDNSRecord::register_record(
			self,
			flags,
			interface_index,
			fullname,
			rr_type,
			rr_class,
			rdata,
			ttl,
			callback,
			context,
		)?))
	}

	pub(crate) fn create_connection() -> FFIResult<DNSService> {
		Self::new(InnerDNSService::create_connection())
	}

	pub(crate) fn query_record(
		flags: ffi::DNSServiceFlags,
		interface_index: u32,
		fullname: &cstr::CStr<'_>,
		rr_type: Type,
		rr_class: Class,
		callback: ffi::DNSServiceQueryRecordReply,
		context: *mut c_void,
	) -> FFIResult<DNSService> {
		Self::new(InnerDNSService::query_record(
			flags,
			interface_index,
			fullname,
			rr_type,
			rr_class,
			callback,
			context,
		))
	}
}

struct InnerDNSRecord(DNSService, ffi::DNSRecordRef, Type);

impl Drop for InnerDNSRecord {
	fn drop(&mut self) {
		if !self.1.is_null() {
			unsafe {
				ffi::DNSServiceRemoveRecord(
					self.get_service().0,
					self.1,
					0, // no flags
				);
			}
		}
	}
}

impl InnerDNSRecord {
	fn get_service(&self) -> &InnerDNSService {
		self.0.get()
	}

	fn rr_type(&self) -> Type {
		self.2
	}

	// only valid when `service` was created through "register"
	fn add_record(
		service: &DNSService,
		flags: ffi::DNSServiceFlags,
		rr_type: Type,
		rdata: &[u8],
		ttl: u32,
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
				rr_type.0,
				rd_len,
				rdata,
				ttl,
			)
		})?;
		Ok(InnerDNSRecord(service.clone(), record_ref, rr_type))
	}

	// only valid when `service` was created through "create_connection"
	fn register_record(
		service: &DNSService,
		flags: ffi::DNSServiceFlags,
		interface_index: u32,
		fullname: &cstr::CStr<'_>,
		rr_type: Type,
		rr_class: Class,
		rdata: &[u8],
		ttl: u32,
		callback: ffi::DNSServiceRegisterRecordReply,
		context: *mut c_void,
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
				rr_type.0,
				rr_class.0,
				rd_len,
				rdata,
				ttl,
				callback,
				context,
			)
		})?;
		Ok(InnerDNSRecord(service.clone(), record_ref, rr_type))
	}

	fn update_record(
		&self,
		flags: ffi::DNSServiceFlags,
		rdata: &[u8],
		ttl: u32,
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
				ttl,
			)
		})
	}

	fn keep(mut self) {
		self.1 = null_mut();
	}
}

pub struct DNSRecord(InnerDNSRecord);

impl DNSRecord {
	pub fn rr_type(&self) -> Type {
		self.0.rr_type()
	}

	pub fn update_record(
		&self,
		flags: ffi::DNSServiceFlags,
		rdata: &[u8],
		ttl: u32,
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
