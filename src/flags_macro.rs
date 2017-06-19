#![macro_use]

macro_rules! flags_ops {
	($flagset:ident: $ty:ty: $flags:ident: $($case:ident,)*) => (
		impl ::std::ops::BitOr<$flags> for $flags {
			type Output = $flagset;
			fn bitor(self, rhs: $flags) -> Self::Output {
				$flagset::from(self) | $flagset::from(rhs)
			}
		}

		impl $flagset {
			/// Construct empty set of flags.
			pub fn none() -> Self {
				$flagset(0)
			}
		}

		impl ::std::fmt::Debug for $flagset {
			fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
				write!(f, "[")?;
				$(
					if *self & $flags::$case {
						write!(f, "{:?},", $flags::$case)?;
					}
				)*
				write!(f, "]")
			}
		}

		impl ::std::default::Default for $flagset {
			fn default() -> Self {
				$flagset(0)
			}
		}

		impl ::std::convert::From<$flags> for $flagset {
			fn from(flag: $flags) -> Self {
				$flagset(1 << (flag as u8))
			}
		}

		impl ::std::convert::Into<$ty> for $flagset {
			fn into(self) -> $ty {
				self.0
			}
		}

		impl ::std::ops::BitOr<$flags> for $flagset {
			type Output = $flagset;
			fn bitor(self, rhs: $flags) -> Self::Output {
				self | $flagset::from(rhs)
			}
		}

		impl ::std::ops::BitOr<$flagset> for $flagset {
			type Output = $flagset;
			fn bitor(self, rhs: $flagset) -> Self::Output {
				$flagset(self.0 | rhs.0)
			}
		}

		impl<T> ::std::ops::BitOrAssign<T> for $flagset
		where $flagset: ::std::ops::BitOr<T, Output=$flagset> {
			fn bitor_assign(&mut self, rhs: T) {
				*self = *self | rhs;
			}
		}

		impl ::std::ops::BitAnd<$flags> for $flagset {
			type Output = bool;
			fn bitand(self, rhs: $flags) -> Self::Output {
				0 != (self.0 & $flagset::from(rhs).0)
			}
		}

		impl ::std::ops::BitAnd<$flagset> for $flags {
			type Output = bool;
			fn bitand(self, rhs: $flagset) -> Self::Output {
				0 != ($flagset::from(self).0 & rhs.0)
			}
		}
	);
}

macro_rules! flag_mapping {
	($flagset:ident: $flags:ident => $ty:ty:
		$($case:ident => $value:expr,)*
	) => (
		impl Into<$ty> for $flagset {
			fn into(self) -> $ty {
				$(
					(if self & $flags::$case {
						$value
					} else {
						0
					})
				|)*
				0
			}
		}

		impl From<$ty> for $flagset {
			fn from(value: $ty) -> Self {
				$(
					(if 0 != value & $value {
						$flags::$case.into()
					} else {
						$flagset::none()
					})
				|)*
				$flagset::none()
			}
		}
	);
}
