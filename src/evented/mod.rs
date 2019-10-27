#[cfg(unix)]
use self::unix as platform;
#[cfg(unix)]
mod unix;

#[cfg(windows)]
use self::windows as platform;
#[cfg(windows)]
mod windows;

use futures;
use std::io;

use raw::DNSService;

#[must_use = "EventedDNSService does nothing unless polled"]
pub struct EventedDNSService {
	service: DNSService,
	poll: platform::PollReadFd,
}

impl EventedDNSService {
	pub fn new(service: DNSService) -> io::Result<Self> {
		let fd = service.fd();

		Ok(EventedDNSService {
			service,
			poll: platform::PollReadFd::new(fd)?,
		})
	}

	pub fn poll(&self) -> io::Result<()> {
		match self.poll.poll_read_ready()? {
			futures::Async::Ready(()) => {
				let fd = self.service.fd();
				while platform::is_readable(fd)? {
					self.service.process_result()?;
				}
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
