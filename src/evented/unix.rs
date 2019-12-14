use futures::{Async, Poll, try_ready};
use mio;
use std::{
	io,
	os::raw::c_int,
};
use tokio::reactor::{
	PollEvented2 as PollEvented,
};

pub fn is_readable(fd: c_int) -> io::Result<bool> {
	let mut fds = libc::pollfd {
		fd,
		events: libc::POLLIN | libc::POLLHUP | libc::POLLERR,
		revents: 0,
	};
	loop {
		let r = unsafe { libc::poll(&mut fds, 1, 0) };
		if r == 0 { return Ok(false); }
		if r == 1 { return Ok(true); }
		let e = io::Error::last_os_error();
		if e.kind() == io::ErrorKind::Interrupted {
			continue
		}
		return Ok(false);
	}
}

pub struct PollReadFd(PollEvented<EventedFd>);

impl PollReadFd {
	pub fn new(fd: c_int) -> io::Result<Self> {
		Ok(PollReadFd(PollEvented::new(EventedFd(fd))))
	}

	pub fn poll_read_ready(&self) -> Poll<(), io::Error> {
		if try_ready!(self.0.poll_read_ready(mio::Ready::readable())).is_readable() {
			Ok(Async::Ready(()))
		} else {
			Ok(Async::NotReady)
		}
	}

	pub fn clear_read_ready(&self) -> io::Result<()> {
		self.0.clear_read_ready(mio::Ready::readable())
	}
}

struct EventedFd(c_int);

impl mio::Evented for EventedFd {
	fn register(
		&self,
		poll: &mio::Poll,
		token: mio::Token,
		interest: mio::Ready,
		opts: mio::PollOpt,
	) -> io::Result<()> {
		let efd = mio::unix::EventedFd(&self.0);
		mio::Evented::register(&efd, poll, token, interest, opts)
	}

	fn reregister(
		&self,
		poll: &mio::Poll,
		token: mio::Token,
		interest: mio::Ready,
		opts: mio::PollOpt,
	) -> io::Result<()> {
		let efd = mio::unix::EventedFd(&self.0);
		mio::Evented::reregister(&efd, poll, token, interest, opts)
	}

	fn deregister(&self, poll: &mio::Poll) -> io::Result<()> {
		let efd = mio::unix::EventedFd(&self.0);
		mio::Evented::deregister(&efd, poll)
	}
}
