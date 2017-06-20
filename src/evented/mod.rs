#[cfg(unix)]
use self::unix::EventedFd;
#[cfg(unix)]
mod unix;

use futures;
use std::io;
use tokio_core::reactor::{Handle,PollEvented,Remote};

use raw::DNSService;
use remote::GetRemote;

pub struct EventedDNSService {
	service: DNSService,
	poll: PollEvented<EventedFd>,
}

impl EventedDNSService {
	pub fn new(service: DNSService, handle: &Handle) -> io::Result<Self> {
		let fd = service.fd();

		Ok(EventedDNSService{
			service: service,
			poll: PollEvented::new(EventedFd::new(fd)?, handle)?,
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
