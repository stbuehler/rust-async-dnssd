use std::fmt;

pub fn hex_dump_bytes(buf: &[u8], f: &mut fmt::Formatter) -> fmt::Result {
	if buf.is_empty() {
		return write!(f, "[]");
	}
	let addr_len = (0usize.leading_zeros() - buf.len().leading_zeros()) as usize;
	let addr_len = ((addr_len + 7) & !7) / 4;
	write!(f, "[ /* {} bytes */\n", buf.len())?;
	for (i, line) in buf.chunks(16).enumerate() {
		write!(f, "0x{:0width$x}:", i * 16, width = addr_len)?;
		for part in line.chunks(8) {
			for c in part {
				write!(f, " {:02x}", c)?;
			}
			write!(f, " ")?;
		}
		write!(f, "{:width$}  ", "", width = 3 * (16 - line.len()) + (16 - line.len()) / 8 )?;
		for c in line {
			write!(f, "{}", if *c >= 0x20 && *c <= 0x7e { *c as char } else { '.' })?;
		}
		write!(f, "\n")?;
	}
	write!(f, "0x{:0width$x}]", buf.len(), width = addr_len)?;
	Ok(())
}

pub struct HexDisplay<T>(pub T);

impl<T: AsRef<[u8]>> fmt::Debug for HexDisplay<T> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		hex_dump_bytes(self.0.as_ref(), f)
	}
}

impl<T: AsRef<[u8]>> fmt::Display for HexDisplay<T> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		hex_dump_bytes(self.0.as_ref(), f)
	}
}
