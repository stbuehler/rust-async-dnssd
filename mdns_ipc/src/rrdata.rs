use bytes::{self,BufMut};
use mdns_ipc_core;
use std::io;
use std::fmt;

use errors::*;

#[derive(Clone,PartialEq,Eq,PartialOrd,Ord,Hash)]
pub struct RRData(pub bytes::Bytes);

impl mdns_ipc_core::Serialize for RRData {
	fn serialized_size(&self) -> io::Result<usize> {
		Ok(2 + self.0.len())
	}

	fn serialize(&self, dest: &mut bytes::BytesMut) -> io::Result<()> {
		if self.0.len() > 65535 {
			return Err(RRDataTooLong.into());
		}

		let len = self.0.len() as u16;
		len.serialize(dest)?;
		dest.put(&*self.0);
		Ok(())
	}
}

impl mdns_ipc_core::Deserialize for RRData {
	fn deserialize(src: &mut io::Cursor<bytes::Bytes>) -> io::Result<Self> {
		let len = u16::deserialize(src)?;
		Ok(RRData(mdns_ipc_core::extract_slice(src, len as usize)?))
	}

	fn deserialize_size_hint() -> mdns_ipc_core::SizeHint {
		mdns_ipc_core::SizeHint::AtLeast(2)
	}
}

impl fmt::Debug for RRData {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		mdns_ipc_core::hex_dump_bytes(self.0.as_ref(), f)
	}
}

impl fmt::Display for RRData {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		mdns_ipc_core::hex_dump_bytes(self.0.as_ref(), f)
	}
}
