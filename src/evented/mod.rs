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
use tokio_core::reactor::{
	Handle,
	Remote,
};

use raw::DNSService;
use remote::GetRemote;

#[must_use = "EventedDNSService does nothing unless polled"]
pub struct EventedDNSService {
	service: DNSService,
	poll: PollReadFd,
}

impl EventedDNSService {
	pub fn new(service: DNSService, handle: &Handle) -> io::Result<Self> {
		let fd = service.fd();

		Ok(EventedDNSService {
			service,
			poll: PollReadFd::new(fd, handle)?,
		})
	}

	pub fn poll(&self) -> io::Result<()> {
		match self.poll.poll_read() {
			futures::Async::Ready(()) => {
				self.service.process_result()?;
				self.poll.need_read();
			},
			futures::Async::NotReady => (),
		}
		Ok(())
	}

	pub fn service(&self) -> &DNSService {
		&self.service
	}
}

impl GetRemote for EventedDNSService {
	fn remote(&self) -> &Remote {
		self.poll.remote()
	}
}
