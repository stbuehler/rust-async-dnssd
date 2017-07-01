use bytes::{self,BufMut,Buf};
use std::io;

use errors::*;
use traits::*;
use utils::*;

fn extract_zero_terminated(src: &mut io::Cursor<bytes::Bytes>) -> io::Result<bytes::Bytes> {
	let len = match src.bytes().iter().position(|b| *b == 0) {
		Some(ndx) => ndx,
		// need at least one more byte...
		None => return Err(NeedMoreData(1).into()),
	};
	let result = extract_slice(src, len)?;
	let zero = <u8 as Deserialize>::deserialize(src)?;
	assert_eq!(zero, 0);
	Ok(result)
}

fn write_zero_terminated(dest: &mut bytes::BytesMut, data: &[u8]) -> io::Result<()> {
	if data.iter().any(|b| *b == 0) {
		return Err(StringContainsNull.into())
	}
	dest.reserve(data.len() + 1);
	dest.put(data);
	dest.put_u8(0);
	Ok(())
}

/// collection of unicode code points which are all < 256
///
/// ASCII is a subset
pub struct Latin1String(pub bytes::Bytes);

impl Latin1String {
	pub fn new<T: AsRef<[u8]>>(data: T) -> Self {
		Latin1String(bytes::Bytes::from(data.as_ref()))
	}

	pub fn bytes(&self) -> &[u8] {
		&*self.0
	}
}

impl AsRef<[u8]> for Latin1String {
	fn as_ref(&self) -> &[u8] {
		self.bytes()
	}
}

impl Serialize for Latin1String {
	fn serialized_size(&self) -> io::Result<usize> {
		Ok(self.bytes().len() + 1)
	}

	fn serialize(&self, dest: &mut bytes::BytesMut) -> io::Result<()> {
		write_zero_terminated(dest, self.bytes())
	}
}

impl Deserialize for Latin1String {
	fn deserialize(src: &mut io::Cursor<bytes::Bytes>) -> io::Result<Self> {
		Ok(Latin1String(extract_zero_terminated(src)?))
	}

	fn deserialize_size_hint() -> SizeHint {
		SizeHint::AtLeast(1)
	}
}

pub enum Latin1Str {
	Static(&'static [u8]),
	Shared(Latin1String),
}

impl Latin1Str {
	pub fn new<T: AsRef<[u8]>>(data: T) -> Self {
		Latin1Str::Shared(Latin1String::new(data))
	}

	pub fn bytes(&self) -> &[u8] {
		match *self {
			Latin1Str::Static(ref s) => s,
			Latin1Str::Shared(ref s) => s.bytes(),
		}
	}
}

impl AsRef<[u8]> for Latin1Str {
	fn as_ref(&self) -> &[u8] {
		self.bytes()
	}
}

impl Serialize for Latin1Str {
	fn serialized_size(&self) -> io::Result<usize> {
		Ok(self.bytes().len() + 1)
	}

	fn serialize(&self, dest: &mut bytes::BytesMut) -> io::Result<()> {
		write_zero_terminated(dest, self.bytes())
	}
}

impl Deserialize for Latin1Str {
	fn deserialize(src: &mut io::Cursor<bytes::Bytes>) -> io::Result<Self> {
		Ok(Latin1Str::Shared(Deserialize::deserialize(src)?))
	}

	fn deserialize_size_hint() -> SizeHint {
		SizeHint::AtLeast(1)
	}
}

pub struct Utf8String(bytes::Bytes);

impl Utf8String {
	pub fn new<S: AsRef<str>>(s: S) -> Self {
		Utf8String(bytes::Bytes::from(s.as_ref().as_bytes()))
	}

	pub fn as_str(&self) -> &str {
		self.as_ref()
	}
}

impl ToString for Utf8String {
	fn to_string(&self) -> String {
		self.as_str().to_string()
	}
}

impl AsRef<[u8]> for Utf8String {
	fn as_ref(&self) -> &[u8] {
		self.0.as_ref()
	}
}

impl AsRef<str> for Utf8String {
	fn as_ref(&self) -> &str {
		use std::str::from_utf8_unchecked;
		unsafe { from_utf8_unchecked(self.0.as_ref()) }
	}
}

impl Serialize for Utf8String {
	fn serialized_size(&self) -> io::Result<usize> {
		Ok(self.0.len() + 1)
	}

	fn serialize(&self, dest: &mut bytes::BytesMut) -> io::Result<()> {
		write_zero_terminated(dest, self.as_ref())
	}
}

impl Deserialize for Utf8String {
	fn deserialize(src: &mut io::Cursor<bytes::Bytes>) -> io::Result<Self> {
		use std::str::from_utf8;
		let data = extract_zero_terminated(src)?;
		from_utf8(data.as_ref()).map_err(
			|e| io::Error::new(io::ErrorKind::InvalidData, e)
		)?;
		Ok(Utf8String(data))
	}

	fn deserialize_size_hint() -> SizeHint {
		SizeHint::AtLeast(1)
	}
}

pub enum Utf8Str {
	Static(&'static str),
	Shared(Utf8String),
	Owned(String),
}

impl Utf8Str {
	pub fn new<S: AsRef<str>>(s: S) -> Self {
		Utf8Str::Shared(Utf8String::new(s))
	}

	pub fn as_str(&self) -> &str {
		self.as_ref()
	}

	pub fn into_owned(self) -> String {
		match self {
			Utf8Str::Static(s) => String::from(s),
			Utf8Str::Shared(s) => s.to_string(),
			Utf8Str::Owned(s) => s,
		}
	}
}

impl ToString for Utf8Str {
	fn to_string(&self) -> String {
		self.as_str().to_string()
	}
}

impl AsRef<[u8]> for Utf8Str {
	fn as_ref(&self) -> &[u8] {
		match *self {
			Utf8Str::Static(s) => s.as_ref(),
			Utf8Str::Shared(ref s) => s.as_ref(),
			Utf8Str::Owned(ref s) => s.as_ref(),
		}
	}
}

impl AsRef<str> for Utf8Str {
	fn as_ref(&self) -> &str {
		match *self {
			Utf8Str::Static(s) => s.as_ref(),
			Utf8Str::Shared(ref s) => s.as_ref(),
			Utf8Str::Owned(ref s) => s.as_ref(),
		}
	}
}

impl Serialize for Utf8Str {
	fn serialized_size(&self) -> io::Result<usize> {
		Ok(self.as_str().len() + 1)
	}

	fn serialize(&self, dest: &mut bytes::BytesMut) -> io::Result<()> {
		write_zero_terminated(dest, self.as_ref())
	}
}

impl Deserialize for Utf8Str {
	fn deserialize(src: &mut io::Cursor<bytes::Bytes>) -> io::Result<Self> {
		use std::str::from_utf8;
		let data = extract_zero_terminated(src)?;
		from_utf8(data.as_ref()).map_err(
			|e| io::Error::new(io::ErrorKind::InvalidData, e)
		)?;
		Ok(Utf8Str::Shared(Deserialize::deserialize(src)?))
	}

	fn deserialize_size_hint() -> SizeHint {
		SizeHint::AtLeast(1)
	}
}
