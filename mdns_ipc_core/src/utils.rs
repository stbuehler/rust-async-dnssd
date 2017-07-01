use bytes;
use std::io;

use errors::*;

pub fn extract_slice(src: &mut io::Cursor<bytes::Bytes>, len: usize) -> Result<bytes::Bytes, NeedMoreData> {
	check_length(src, len)?;
	let pos = src.position() as usize;
	let result = src.get_ref().slice(pos, pos + len);
	bytes::Buf::advance(src, len);
	Ok(result)
}
