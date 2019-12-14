//! On windows we cannot get read (or write) events for sockets in the
//! IOCP model; we only can run asynchronous reads!
//!
//! So we need to use select() to poll for read, and run it in a
//! separate thread.  To cancel that select() we'd need another socket
//! to select() for, and there is no socketpair() - we could use a
//! loopback TCP connection, but a firewall might block it.
//!
//! Instead we use a small (1 second) timeout for the select; it is only
//! used to terminate the thread anyway.
//!
//! This of course wastes one thread per fd we want to watch; a bigger
//! solution would reuse the same backend thread over and over, but then
//! we'd have to try the loopback TCP connection to wake it and fall
//! back to a smaller timeout.

use futures::{
	sink::Wait,
	sync::mpsc as futures_mpsc,
	Async,
	Poll,
	Sink,
	Stream,
};
use std::{
	cell::UnsafeCell,
	io,
	os::raw::c_int,
	sync::mpsc as std_mpsc,
	thread,
	time::Duration,
};
use winapi::um::winsock2;

use self::fd_set::FdSet;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum PollRequest {
	Poll,
	Close,
}

struct SelectFdRead {
	fd: c_int,
	read_fds: FdSet,
}
impl SelectFdRead {
	pub fn new(fd: c_int) -> Self {
		SelectFdRead { fd, read_fds: FdSet::new() }
	}

	pub fn select(&mut self, timeout: Option<Duration>) -> bool {
		use std::ptr::null_mut;
		let mut timeout = timeout.map(|timeout| winsock2::timeval {
			tv_sec: timeout.as_secs() as libc::c_long,
			tv_usec: (timeout.subsec_nanos() / 1000) as libc::c_long,
		});
		self.read_fds.set(self.fd);
		unsafe {
			winsock2::select(
				self.fd + 1,
				self.read_fds.inner(),
				null_mut(),
				null_mut(),
				timeout.as_mut().map(|x| x as *mut _).unwrap_or(null_mut()),
			);
		}
		self.read_fds.is_set(self.fd)
	}
}

struct Inner {
	/// file descriptor to watch read events for
	fd: c_int,
	/// background select thread
	_thread: thread::JoinHandle<()>,
	/// either the select thread is running a Poll request or we manually
	/// sent a response through `send_response`
	pending_request: bool,
	/// send poll or close request to select thread
	send_request: std_mpsc::SyncSender<PollRequest>,
	/// when clear_read_ready() is called we use this to trigger a response if
	/// we already know the read event is pending
	send_response: Wait<futures_mpsc::Sender<()>>,
	/// a response means a read event is pending
	recv_response: futures_mpsc::Receiver<()>,
}

impl Inner {
	fn poll_read_ready(&mut self) -> Poll<(), io::Error> {
		debug!("poll read");
		if !self.pending_request {
			let mut read_fds = SelectFdRead::new(self.fd);
			if read_fds.select(Some(Duration::from_millis(0))) {
				debug!("poll read: local ready");
				return Ok(Async::Ready(()));
			} else {
				debug!("poll read: not ready, start thread");
				self.send_request
					.send(PollRequest::Poll)
					.expect("select thread terminated");
				self.pending_request = true;
			}
		}

		match self.recv_response.poll().unwrap() {
			Async::Ready(None) => unreachable!(),
			Async::Ready(Some(())) => {
				debug!("poll read: thread ready");
				self.pending_request = false;
				Ok(Async::Ready(()))
			},
			Async::NotReady => {
				debug!("poll read: thread not ready");
				Ok(Async::NotReady)
			},
		}
	}

	fn clear_read_ready(&mut self) -> io::Result<()> {
		// we need to get Async::NotReady from recv_response.poll
		match self.recv_response.poll().unwrap() {
			Async::Ready(None) => unreachable!(),
			Async::Ready(Some(())) => {
				// was ready. damn...
				assert!(self.pending_request);
				// try again - can't be ready again
				match self.recv_response.poll().unwrap() {
					Async::Ready(None) => unreachable!(),
					Async::Ready(Some(())) => unreachable!(),
					Async::NotReady => (),
				}
				// now send a response - it was ready after all
				self.send_response.send(()).unwrap();
			},
			Async::NotReady => {
				// yay!
				//
				// now we need something to trigger a response
				let mut read_fds = SelectFdRead::new(self.fd);
				self.pending_request = true;
				if read_fds.select(Some(Duration::from_millis(0))) {
					// ready, send a response
					self.send_response.send(()).unwrap();
				} else {
					debug!("poll need read: not ready, start thread");
					self.send_request
						.send(PollRequest::Poll)
						.expect("select thread terminated");
				}
			},
		}
		Ok(())
	}
}

pub fn is_readable(fd: c_int) -> io::Result<bool> {
	let mut read_fds = SelectFdRead::new(fd);
	Ok(read_fds.select(Some(Duration::from_millis(0))))
}

pub struct PollReadFd(UnsafeCell<Inner>);
impl PollReadFd {
	/// does not take overship of fd
	pub fn new(fd: c_int) -> io::Result<Self> {
		// buffer one request for "Close"
		let (send_request, recv_request) = std_mpsc::sync_channel(1);
		// buffer one notification
		let (send_response, recv_response) = futures_mpsc::channel(1);
		let outer_send_response = send_response.clone().wait();

		let thread = thread::spawn(move || {
			let mut read_fds = SelectFdRead::new(fd);
			let mut send_response = send_response.wait();
			loop {
				debug!("[select thread] waiting for request");
				match recv_request.recv() {
					Ok(PollRequest::Poll) => (),
					Ok(PollRequest::Close) => return,
					Err(_) => return,
				}
				debug!("[select thread] start polling");

				while !read_fds.select(Some(Duration::from_millis(1000))) {
					match recv_request.try_recv() {
						Ok(PollRequest::Poll) => unreachable!(),
						Ok(PollRequest::Close) => return,
						Err(_) => (), // back to select()
					}
				}

				debug!("[select thread] read event");

				if send_response.send(()).is_err() {
					return;
				}
			}
		});

		Ok(PollReadFd(UnsafeCell::new(Inner {
			fd,
			_thread: thread,
			pending_request: false,
			send_request,
			send_response: outer_send_response,
			recv_response,
		})))
	}

	fn inner(&self) -> &mut Inner {
		unsafe { &mut *self.0.get() }
	}

	pub fn poll_read_ready(&self) -> Poll<(), io::Error> {
		self.inner().poll_read_ready()
	}

	pub fn clear_read_ready(&self) -> io::Result<()> {
		self.inner().clear_read_ready()
	}
}

impl Drop for PollReadFd {
	fn drop(&mut self) {
		let _ = self.inner().send_request.send(PollRequest::Close);
	}
}

mod fd_set {
	use libc::{c_int, c_uint};
	use winapi::um::winsock2::{
		fd_set as RawFdSet,
		FD_SETSIZE,
		SOCKET,
	};
	use std::mem::MaybeUninit;

	pub(super) struct FdSet(RawFdSet);

	impl FdSet {
		pub fn new() -> Self {
			let fd_count = 0;
			// This is safe because we don't read from it when the slot is not initialized.
			let fd_array = [unsafe {
				MaybeUninit::<usize>::uninit().assume_init()
			}; FD_SETSIZE];
			FdSet(RawFdSet { fd_count, fd_array })
		}

		pub fn inner(&mut self) -> *mut RawFdSet {
			&mut self.0
		}

		pub fn set(&mut self, fd: c_int) {
			if self.is_set(fd) {
				return;
			}
			let count = self.0.fd_count as usize;
			if count < FD_SETSIZE {
				self.0.fd_array[count] = fd as c_uint as SOCKET;
				self.0.fd_count += 1;
			}
		}

		pub fn is_set(&self, fd: c_int) -> bool {
			let fd = fd as c_uint as SOCKET;
			let count = self.0.fd_count as usize;
			self.0.fd_array[..count].iter().any(|i| *i == fd)
		}
	}
}
