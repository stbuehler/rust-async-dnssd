use std::io::{Cursor,Result};
use bytes::{Bytes,BytesMut};

use traits::*;
use errors::*;

pub trait ConstCheck<T>: Sized {
	fn deserialize(src: &mut Cursor<Bytes>, const_value: T) -> Result<Self>;
	fn deserialize_size_hint(const_value: T) -> SizeHint;
	fn serialize(dst: &mut BytesMut, const_value: T) -> Result<()>;
	fn serialized_size(const_value: T) -> Result<usize>;
}

#[derive(Clone,Copy,PartialEq,Eq,PartialOrd,Ord,Hash,Debug,Default)]
pub struct Const;

impl<T: Serialize+Deserialize+Eq> ConstCheck<T> for Const {
	fn deserialize(src: &mut Cursor<Bytes>, const_value: T) -> Result<Self> {
		let val = <T as Deserialize>::deserialize(src)?;
		if val == const_value {
			Ok(Const)
		} else {
			Err(UnexpectedConstValue.into())
		}
	}

	fn deserialize_size_hint(_const_value: T) -> SizeHint {
		<T as Deserialize>::deserialize_size_hint()
	}

	fn serialize(dst: &mut BytesMut, const_value: T) -> Result<()> {
		Serialize::serialize(&const_value, dst)
	}

	fn serialized_size(const_value: T) -> Result<usize> {
		Serialize::serialized_size(&const_value)
	}
}
