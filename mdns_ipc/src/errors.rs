use std::error;
use std::fmt;
use std::io;

macro_rules! simple_error {
	($name:ident, $kind:ident, $desc:expr) => (
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

simple_error!{RRDataTooLong, InvalidData, "RRData too long"}
simple_error!{ErrorCodeNotAnError, InvalidData, "Error code is not an error"}
