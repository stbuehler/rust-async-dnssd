use std::io;

use raw;

pub struct Record(raw::DNSRecord);

impl Record {
	pub fn rr_type(&self) -> u16 {
		self.0.rr_type()
	}

	pub fn update_raw_record(
		&self,
		rdata: &[u8],
		ttl: u32
	) -> io::Result<()> {
		self.0.update_record(
			0, /* no flags */
			rdata,
			ttl
		)?;
		Ok(())
	}

	// keep "forever" (until service is dropped)
	pub fn keep(self) {
		self.0.keep()
	}
}

pub fn new_record(r: raw::DNSRecord) -> Record {
	Record(r)
}
