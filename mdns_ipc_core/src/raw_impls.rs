use std::mem::size_of;
use bytes::{self,BufMut,Buf};
use std::io;

use traits::*;
use errors::*;

macro_rules! byte_int {
	($ty:ty : $put:ident : $get:ident) => (
		impl Serialize for $ty {
			fn serialized_size(&self) -> io::Result<usize> {
				Ok(size_of::<Self>())
			}

			fn serialize(&self, dest: &mut bytes::BytesMut) -> io::Result<()> {
				dest.reserve(1);
				dest.$put(*self);
				Ok(())
			}
		}

		impl Deserialize for $ty {
			fn deserialize(src: &mut io::Cursor<bytes::Bytes>) -> io::Result<Self> {
				check_length(src, size_of::<Self>())?;
				Ok(src.$get())
			}

			fn deserialize_size_hint() -> SizeHint {
				SizeHint::Exact(size_of::<Self>())
			}
		}
	)
}

macro_rules! big_endian_int {
	($ty:ty : $put:ident : $get:ident) => (
		impl Serialize for $ty {
			fn serialized_size(&self) -> io::Result<usize> {
				Ok(size_of::<Self>())
			}

			fn serialize(&self, dest: &mut bytes::BytesMut) -> io::Result<()> {
				dest.reserve(1);
				dest.$put::<bytes::BigEndian>(*self);
				Ok(())
			}
		}

		impl Deserialize for $ty {
			fn deserialize(src: &mut io::Cursor<bytes::Bytes>) -> io::Result<Self> {
				check_length(src, size_of::<Self>())?;
				Ok(src.$get::<bytes::BigEndian>())
			}

			fn deserialize_size_hint() -> SizeHint {
				SizeHint::Exact(size_of::<Self>())
			}
		}
	)
}

byte_int!{u8 : put_u8 : get_u8}
big_endian_int!{u16 : put_u16 : get_u16}
big_endian_int!{u32 : put_u32 : get_u32}
big_endian_int!{u64 : put_u64 : get_u64}
byte_int!{i8 : put_i8 : get_i8}
big_endian_int!{i16 : put_i16 : get_i16}
big_endian_int!{i32 : put_i32 : get_i32}
big_endian_int!{i64 : put_i64 : get_i64}

impl Serialize for () {
	fn serialized_size(&self) -> io::Result<usize> {
		Ok(0)
	}

	fn serialize(&self, _dest: &mut bytes::BytesMut) -> io::Result<()> {
		Ok(())
	}
}

impl Deserialize for () {
	fn deserialize(_src: &mut io::Cursor<bytes::Bytes>) -> io::Result<Self> {
		Ok(())
	}

	fn deserialize_size_hint() -> SizeHint {
		SizeHint::Exact(0)
	}
}
