use bytes;
use std::io;
use std::ops::Add;

use errors::*;

pub fn parse<T: Deserialize>(src: bytes::Bytes) -> io::Result<T> {
	let mut src = ::std::io::Cursor::new(src);

	let result = <T as Deserialize>::deserialize(&mut src)?;

	if src.position() < (src.get_ref().len() as u64) {
		Err(ExpectedEndOfFile.into())
	} else {
		Ok(result)
	}
}

pub trait Serialize {
	fn serialized_size(&self) -> io::Result<usize>;
	fn serialize(&self, dest: &mut bytes::BytesMut) -> io::Result<()>;
}

pub trait Deserialize: Sized {
	fn deserialize(src: &mut io::Cursor<bytes::Bytes>) -> io::Result<Self>;
	fn deserialize_size_hint() -> SizeHint;
}

#[derive(Clone,Copy,PartialEq,Eq,Hash,Debug)]
pub enum SizeHint {
	Exact(usize),
	AtLeast(usize),
}

impl Add<SizeHint> for SizeHint {
	type Output = SizeHint;
	fn add(self, rhs: SizeHint) -> Self::Output {
		use SizeHint::*;
		match (self, rhs) {
			(Exact(a), Exact(b)) => Exact(a+b),
			(AtLeast(a), Exact(b)) => AtLeast(a+b),
			(Exact(a), AtLeast(b)) => AtLeast(a+b),
			(AtLeast(a), AtLeast(b)) => AtLeast(a+b),
		}
	}
}
