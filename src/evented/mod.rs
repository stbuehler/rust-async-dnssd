#[cfg(unix)]
use self::unix::*;
#[cfg(unix)]
mod unix;

#[cfg(windows)]
use self::windows::*;
#[cfg(windows)]
mod windows;

use futures;
use std::io;

use raw::DNSService;

#[must_use = "EventedDNSService does nothing unless polled"]
pub struct EventedDNSService {
	service: DNSService,
	poll: PollReadFd,
}

impl EventedDNSService {
	pub fn new(service: DNSService) -> io::Result<Self> {
		let fd = service.fd();

		Ok(EventedDNSService {
			service,
			poll: PollReadFd::new(fd)?,
		})
	}

	pub fn poll(&self) -> io::Result<()> {
		match self.poll.poll_read_ready()? {
			futures::Async::Ready(()) => {
				self.service.process_result()?;
				self.poll.clear_read_ready()?;
			},
			futures::Async::NotReady => (),
		}
		Ok(())
	}

	pub fn service(&self) -> &DNSService {
		&self.service
	}
}
