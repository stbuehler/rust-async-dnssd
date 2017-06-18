use futures;
use std::os::raw::{c_int};
use mio::unix::EventedFd;
use std::io;
use tokio_core::reactor::{Handle,PollEvented,Remote};

use raw::DNSService;
use remote::GetRemote;

struct Inner {
	service: DNSService,
	fd: Box<c_int>,
	poll: PollEvented<EventedFd<'static>>,
}

pub struct EventedDNSService(Option<Inner>);

impl EventedDNSService {
	fn inner(&self) -> &Inner {
		self.0.as_ref().expect("EventedDNSService already dropped")
	}

	pub fn new(service: DNSService, handle: &Handle) -> io::Result<Self> {
		let fd = Box::new(service.fd());
		let fd_ref = unsafe {
			// implement Drop manually to ensure fd is dropped
			// after poll
			::std::mem::transmute::<&c_int, &'static c_int>(&*fd)
		};

		Ok(EventedDNSService(Some(Inner{
			service: service,
			fd: fd,
			poll: PollEvented::new(EventedFd(fd_ref), handle)?,
		})))
	}

	pub fn poll(&self) -> io::Result<()> {
		let inner = self.inner();
		match inner.poll.poll_read() {
			futures::Async::Ready(()) => {
				inner.service.process_result()?;
				inner.poll.need_read();
			},
			futures::Async::NotReady => (),
		}
		Ok(())
	}

	pub fn service(&self) -> &DNSService {
		&self.inner().service
	}
}

impl GetRemote for EventedDNSService {
	fn remote(&self) -> &Remote {
		self.inner().poll.remote()
	}
}

impl Drop for EventedDNSService {
	fn drop(&mut self) {
		let i = self.0.take().expect("EventedDNSService already dropped");
		//// make sure to drop poll before (boxed) fd
		drop(i.poll);
		drop(i.fd);
		drop(i.service);
	}
}
