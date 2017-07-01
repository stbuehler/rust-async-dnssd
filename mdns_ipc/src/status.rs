use bytes;
use mdns_ipc_core::{Serialize,Deserialize,SizeHint};
use std::io;

use enums::*;
use errors::*;

pub enum Status<Data = ()> {
	Ok(Data),
	Err(ErrorCode),
}

impl<Data: Serialize> Serialize for Status<Data> {
	fn serialized_size(&self) -> io::Result<usize> {
		Ok(match *self {
			Status::Ok(ref d) =>
				ERROR_NO_ERROR.serialized_size()?
				+ d.serialized_size()?,
			Status::Err(e) => e.serialized_size()?,
		})
	}

	fn serialize(&self, dest: &mut bytes::BytesMut) -> io::Result<()> {
		match *self {
			Status::Ok(ref d) => {
				ERROR_NO_ERROR.serialize(dest)?;
				d.serialize(dest)
			},
			Status::Err(e) => {
				if e == ERROR_NO_ERROR {
					return Err(ErrorCodeNotAnError.into());
				}
				e.serialize(dest)
			},
		}
	}
}

impl<Data: Deserialize> Deserialize for Status<Data> {
	fn deserialize(src: &mut io::Cursor<bytes::Bytes>) -> io::Result<Self> {
		let e = ErrorCode::deserialize(src)?;
		if e == ERROR_NO_ERROR {
			Ok(Status::Ok(Data::deserialize(src)?))
		} else {
			Ok(Status::Err(e))
		}
	}

	fn deserialize_size_hint() -> SizeHint {
		ErrorCode::deserialize_size_hint() + match Data::deserialize_size_hint() {
			SizeHint::Exact(n) => SizeHint::AtLeast(n),
			SizeHint::AtLeast(n) => SizeHint::AtLeast(n),
		}
	}
}
