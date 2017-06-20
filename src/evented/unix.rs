use std::os::raw::{c_int};
use mio;
use std::io;

pub struct EventedFd(c_int);
impl EventedFd {
	/// does not take overship of fd
	pub fn new(fd: c_int) -> io::Result<Self> {
		Ok(EventedFd(fd))
	}
}

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
