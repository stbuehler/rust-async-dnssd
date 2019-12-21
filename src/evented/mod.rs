#[cfg(unix)]
use self::unix as platform;
#[cfg(unix)]
mod unix;

#[cfg(windows)]
use self::windows as platform;
#[cfg(windows)]
mod windows;

use std::{
	io,
	task::{
		Context,
		Poll,
	},
};

pub(crate) struct ReadProcessor {
	fd: libc::c_int,
	poll: platform::PollReadFd,
}

impl ReadProcessor {
	pub(crate) fn new(fd: libc::c_int) -> io::Result<Self> {
		Ok(Self {
			fd,
			poll: platform::PollReadFd::new(fd)?,
		})
	}

	/// call "p" until fd is no longer readable
	pub(crate) fn process<P>(&mut self, cx: &mut Context<'_>, mut p: P) -> io::Result<()>
	where
		P: FnMut() -> io::Result<()>,
	{
		match self.poll.poll_read_ready(cx)? {
			Poll::Ready(()) => {
				while platform::is_readable(self.fd)? {
					p()?;
				}
				self.poll.clear_read_ready(cx)?;
			},
			Poll::Pending => (),
		}
		Ok(())
	}
}
