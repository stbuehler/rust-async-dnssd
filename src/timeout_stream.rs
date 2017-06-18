use futures::{self,Async,Future};
use std::io;
use std::time::Duration;
use tokio_core::reactor::{Timeout,Remote};

use remote::GetRemote;

pub trait TimeoutTrait: futures::Stream+Sized {
	fn timeout(self, duration: Duration) -> io::Result<TimeoutStream<Self>>;
}

impl<S: futures::Stream+GetRemote> TimeoutTrait for S {
	fn timeout(self, duration: Duration) -> io::Result<TimeoutStream<Self>> {
		TimeoutStream::new(self, duration)
	}
}

pub struct TimeoutStream<S> {
	stream: S,
	duration: Duration,
	timeout: Option<Timeout>,
}

impl<S: futures::Stream+GetRemote> TimeoutStream<S> {
	pub fn new(stream: S, duration: Duration) -> io::Result<Self> {
		Ok(TimeoutStream{
			stream: stream,
			duration: duration,
			// delay initialization of timeout, as we cannot get handle
			// from remote outside poll reliably
			timeout: None,
		})
	}
}

#[derive(Debug)]
pub enum TimeoutStreamError<E> {
	StreamError(E),
	TimeoutError(io::Error),
}
impl<E: Into<io::Error>> TimeoutStreamError<E> {
	pub fn into_io_error(self) -> io::Error {
		match self {
			TimeoutStreamError::StreamError(e) => e.into(),
			TimeoutStreamError::TimeoutError(e) => e,
		}
	}
}
impl<S: futures::Stream+GetRemote> TimeoutStream<S> {
	fn reset_timer(&mut self) -> Result<(), TimeoutStreamError<S::Error>> {
		let handle = self.stream.remote().handle().expect("couldn't get handle in poll");
		self.timeout = Some(match Timeout::new(self.duration, &handle) {
			Ok(timeout) => timeout,
			Err(e) => return Err(TimeoutStreamError::TimeoutError(e)),
		});
		Ok(())
	}

	fn get_timer(&mut self) -> Result<&mut Timeout, TimeoutStreamError<S::Error>> {
		if self.timeout.is_none() {
			self.reset_timer()?;
		}
		Ok(self.timeout.as_mut().unwrap())
	}
}


impl<S: futures::Stream+GetRemote> futures::Stream for TimeoutStream<S> {
	type Item = S::Item;
	type Error = TimeoutStreamError<S::Error>;

	fn poll(&mut self) -> Result<Async<Option<Self::Item>>, Self::Error> {
		match self.stream.poll() {
			Ok(Async::Ready(None)) => Ok(Async::Ready(None)), // end of stream
			Ok(Async::Ready(item)) => {
				// not end of stream: reset timeout
				self.reset_timer()?;
				Ok(Async::Ready(item))
			},
			Ok(Async::NotReady) => {
				// check timeout
				match self.get_timer()?.poll() {
					// timed out?
					Ok(Async::Ready(_)) => {
						// not an error
						Ok(Async::Ready(None))
						// Err(TimeoutStreamError::Timeout)
					},
					// still time left
					Ok(Async::NotReady) => Ok(Async::NotReady),
					Err(e) => Err(TimeoutStreamError::TimeoutError(e))
				}
			}
			Err(e) => Err(TimeoutStreamError::StreamError(e)),
		}
	}
}

impl<S: futures::Stream+GetRemote> GetRemote for TimeoutStream<S> {
	fn remote(&self) -> &Remote {
		self.stream.remote()
	}
}
