use std::fmt;
use std::error;
use std::io;

use ffi;

#[derive(Clone,Copy,Eq,PartialEq,Hash)]
pub enum Error {
	KnownError(ffi::DNSServiceError),
	UnknownError(i32),
}
impl Error {
	pub fn from(value: ffi::DNSServiceErrorType) -> Result<(), Error> {
		if let Some(_) = ffi::DNSServiceNoError::try_from(value) {
			Ok(())
		} else {
			match ffi::DNSServiceError::try_from(value) {
				Some(e) => Err(Error::KnownError(e)),
				None => Err(Error::UnknownError(value)),
			}
		}
	}
}

impl From<Error> for io::Error {
	fn from(e: Error) -> Self {
		io::Error::new(io::ErrorKind::Other, e)
	}
}

impl fmt::Debug for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			Error::KnownError(ffi_err) => write!(f, "known error {:?}: {}", ffi_err, ffi_err),
			Error::UnknownError(e) => write!(f, "unknown error code: {:?}", e),
		}
	}
}
impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			Error::KnownError(ffi_err) => write!(f, "{}", ffi_err),
			Error::UnknownError(e) => write!(f, "unknown error code: {:?}", e),
		}
	}
}
impl error::Error for Error {
	fn description(&self) -> &str {
		""
	}
}

impl fmt::Display for ffi::DNSServiceError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", error::Error::description(self))
	}
}
impl error::Error for ffi::DNSServiceError {
	fn description(&self) -> &str {
		use ffi::DNSServiceError::*;
		match *self {
			Unknown               => "unknown error",
			NoSuchName            => "no such name",
			NoMemory              => "out of memory",
			BadParam              => "bad parameter",
			BadReference          => "bad reference",
			BadState              => "bad state",
			BadFlags              => "bad flags",
			Unsupported           => "not supported",
			NotInitialized        => "not initialized",
			NoCache               => "no cache",
			AlreadyRegistered     => "already registered",
			NameConflict          => "name conflict",
			Invalid               => "invalid",
			Incompatible          => "client library incompatible with daemon",
			BadInterfaceIndex     => "bad interface index",
			Refused               => "refused",
			NoSuchRecord          => "no such record",
			NoAuth                => "no auth",
			NoSuchKey             => "no such key",
			NoValue               => "no value",
			BufferTooSmall        => "buffer too small",
		}
	}
}
