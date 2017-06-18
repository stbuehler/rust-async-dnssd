pub use self::browse::*;
pub use self::connection::*;
pub use self::enumerate_domains::*;
pub use self::query_record::*;
pub use self::records::Record;
pub use self::register::*;
pub use self::resolve::*;
use self::records::new_record;

mod browse;
mod connection;
mod enumerate_domains;
mod query_record;
mod records;
mod register;
mod resolve;

pub fn reconfirm_record(
	interface_index: ::interface_index::InterfaceIndex,
	fullname: &str,
	rr_type: u16,
	rr_class: u16,
	rdata: &[u8]
) -> ::std::io::Result<()> {
	let fullname = ::cstr::CStr::from(&fullname)?;
	::raw::reconfirm_record(
		0, /* no flags */
		interface_index.as_raw(),
		&fullname,
		rr_type,
		rr_class,
		rdata);

	Ok(())
}

pub struct FullName<'a> {
	pub service: Option<&'a str>,
	pub reg_type: &'a str,
	pub domain: &'a str,
}

impl<'a> FullName<'a> {
	pub fn construct(&self) -> ::std::io::Result<String> {
		use std::io;

		let service = ::cstr::NullableCStr::from(&self.service)?;
		let reg_type = ::cstr::CStr::from(&self.reg_type)?;
		let domain = ::cstr::CStr::from(&self.domain)?;

		const SIZE : usize = ::ffi::MAX_DOMAIN_NAME + 200;
		let mut buf : Vec<u8> = Vec::new();
		buf.reserve(SIZE);
		let len = unsafe { ::ffi::DNSServiceConstructFullName(
			buf.as_mut_ptr() as *mut i8,
			service.as_ptr(),
			reg_type.as_ptr(),
			domain.as_ptr()
		)};

		if len < 0 {
			return Err(io::Error::new(io::ErrorKind::InvalidInput, "invalid input"))
		}

		unsafe { buf.set_len(len as usize); }

		String::from_utf8(buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))
	}
}
