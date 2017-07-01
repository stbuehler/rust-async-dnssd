use bytes::{self,Buf};
use std::error;
use std::fmt;
use std::io;

macro_rules! simple_error {
	($name:ident, $kind:ident, $desc:expr) => (
		#[derive(PartialEq,Eq)]
		pub struct $name;
		impl fmt::Debug for $name {
			fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
				write!(f, $desc)
			}
		}
		impl fmt::Display for $name {
			fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
				fmt::Debug::fmt(self, f)
			}
		}
		impl error::Error for $name {
			fn description(&self) -> &str {
				$desc
			}
		}
		impl From<$name> for io::Error {
			fn from(e: $name) -> Self {
				io::Error::new(io::ErrorKind::$kind, e)
			}
		}
	)
}

simple_error!{ExpectedEndOfFile, InvalidData, "data was not read completely"}

simple_error!{StringContainsNull, InvalidData, "string contains null byte"}

simple_error!{UnexpectedConstValue, InvalidData, "invalid constant value"}

#[derive(PartialEq,Eq)]
pub struct NeedMoreData(pub usize);
impl fmt::Debug for NeedMoreData {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "not enough data; need {} more bytes", self.0)
	}
}
impl fmt::Display for NeedMoreData {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		fmt::Debug::fmt(self, f)
	}
}
impl error::Error for NeedMoreData {
	fn description(&self) -> &str {
		"not enough data"
	}
}
impl From<NeedMoreData> for io::Error {
	fn from(e: NeedMoreData) -> Self {
		io::Error::new(io::ErrorKind::UnexpectedEof, e)
	}
}

pub fn check_length(src: &io::Cursor<bytes::Bytes>, need: usize) -> Result<(), NeedMoreData> {
	if src.remaining() < need {
		Err(NeedMoreData(need - src.remaining()))
	} else {
		Ok(())
	}
}

pub fn is_need_more_bytes(e: &io::Error) -> Option<usize> {
	if let Some(e) = e.get_ref() {
		if let Some(&NeedMoreData(n)) = e.downcast_ref::<NeedMoreData>() {
			return Some(n);
		}
	}
	None
}
