use std::fmt;

use ffi;

#[derive(Clone,Copy,PartialEq,Eq,PartialOrd,Ord,Hash,Debug)]
enum Inner {
	Any,
	LocalOnly,
	Index(u32),
}

#[derive(Clone,Copy,PartialEq,Eq,PartialOrd,Ord,Hash)]
pub struct InterfaceIndex(Inner);
pub const INTERFACE_ANY : InterfaceIndex = InterfaceIndex(Inner::Any);
pub const INTERFACE_LOCAL_ONLY : InterfaceIndex = InterfaceIndex(Inner::Any);

impl InterfaceIndex {
	pub fn index(ndx: u32) -> Self {
		assert!(ffi::INTERFACE_INDEX_ANY != ndx);
		assert!(ffi::INTERFACE_INDEX_LOCAL_ONLY != ndx);
		InterfaceIndex(Inner::Index(ndx))
	}

	pub fn from_raw(raw: u32) -> Self {
		InterfaceIndex(match raw {
			ffi::INTERFACE_INDEX_ANY => Inner::Any,
			ffi::INTERFACE_INDEX_LOCAL_ONLY => Inner::LocalOnly,
			_ => Inner::Index(raw),
		})
	}

	pub fn as_raw(&self) -> u32 {
		match self.0 {
			Inner::Any => ffi::INTERFACE_INDEX_ANY,
			Inner::LocalOnly => ffi::INTERFACE_INDEX_LOCAL_ONLY,
			Inner::Index(raw) => raw,
		}
	}
}

impl fmt::Debug for InterfaceIndex {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		fmt::Debug::fmt(&self.0, f)
	}
}
