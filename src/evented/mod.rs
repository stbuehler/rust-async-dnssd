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

use crate::raw::DNSService;

#[must_use = "EventedDNSService does nothing unless polled"]
pub(crate) struct EventedDNSService {
	service: DNSService,
	poll: platform::PollReadFd,
}

impl EventedDNSService {
	pub(crate) fn new(service: DNSService) -> io::Result<Self> {
		let fd = service.fd();

		Ok(EventedDNSService {
			service,
			poll: platform::PollReadFd::new(fd)?,
		})
	}

	pub(crate) fn poll(&self) -> io::Result<()> {
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

	pub(crate) fn service(&self) -> &DNSService {
		&self.service
	}
}
