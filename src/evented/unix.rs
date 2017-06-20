use futures::{Async};
use mio;
use std::io;
use std::os::raw::{c_int};
use tokio_core::reactor::{Handle,PollEvented,Remote};

use remote::GetRemote;

pub struct PollReadFd(PollEvented<EventedFd>);

impl PollReadFd {
	pub fn new(fd: c_int, handle: &Handle) -> io::Result<Self> {
		Ok(PollReadFd(PollEvented::new(EventedFd(fd), handle)?))
	}

	pub fn poll_read(&self) -> Async<()> {
		self.0.poll_read()
	}

	pub fn need_read(&self) {
		self.0.need_read()
	}
}

impl GetRemote for PollReadFd {
	fn remote(&self) -> &Remote {
		self.0.remote()
	}
}

struct EventedFd(c_int);

impl mio::Evented for EventedFd {
	fn register(&self, poll: &mio::Poll, token: mio::Token, interest: mio::Ready, opts: mio::PollOpt) -> io::Result<()> {
		let efd = mio::unix::EventedFd(&self.0);
		mio::Evented::register(&efd, poll, token, interest, opts)
	}

	fn reregister(&self, poll: &mio::Poll, token: mio::Token, interest: mio::Ready, opts: mio::PollOpt) -> io::Result<()> {
		let efd = mio::unix::EventedFd(&self.0);
		mio::Evented::reregister(&efd, poll, token, interest, opts)
	}

	fn deregister(&self, poll: &mio::Poll) -> io::Result<()> {
		let efd = mio::unix::EventedFd(&self.0);
		mio::Evented::deregister(&efd, poll)
	}
}
