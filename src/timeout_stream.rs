use futures::{
	self,
	Async,
	Future,
};
use std::{
	io,
	time::Duration,
};

/// `futures::Stream` extension to simplify building
/// [`TimeoutStream`](struct.TimeoutStream.html)
pub trait StreamTimeoutExt: futures::Stream + Sized {
	/// Create new [`TimeoutStream`](struct.TimeoutStream.html)
	fn timeout(self, duration: Duration) -> io::Result<TimeoutStream<Self>>;
}

impl<S: futures::Stream> StreamTimeoutExt for S {
	fn timeout(self, duration: Duration) -> io::Result<TimeoutStream<Self>> {
		TimeoutStream::new(self, duration)
	}
}

/// Add a timeout to a stream; each time an item is received the timer
/// is reset
///
/// If the timeout triggers the stream ends (without an error).
#[must_use = "streams do nothing unless polled"]
pub struct TimeoutStream<S> {
	stream: S,
	duration: Duration,
	timeout: tokio::timer::Delay,
}

impl<S: futures::Stream> TimeoutStream<S> {
	/// Create new `TimeoutStream`.
	///
	/// Also see [`StreamTimeoutExt::timeout`](trait.StreamTimeoutExt.html#method.timeout).
	pub fn new(stream: S, duration: Duration) -> io::Result<Self> {
		Ok(TimeoutStream {
			stream,
			duration,
			timeout: tokio::timer::Delay::new(std::time::Instant::now() + duration),
		})
	}
}

/// Error produces by [`TimeoutStream`](struct.TimeoutStream.html)
///
/// A timeout itself doesn't produce an error, it will just end the
/// stream.
#[derive(Debug)]
pub enum TimeoutStreamError<E> {
	/// An error occured in the underlying stream
	StreamError(E),
	/// Setting / checking the timeout failed
	TimeoutError(tokio::timer::Error),
}

impl<E: Into<io::Error>> TimeoutStreamError<E> {
	/// Combine to an `std::io::Error`.
	pub fn into_io_error(self) -> io::Error {
		match self {
			TimeoutStreamError::StreamError(e) => e.into(),
			TimeoutStreamError::TimeoutError(e) => io::Error::new(io::ErrorKind::Other, e),
		}
	}
}

impl<S: futures::Stream> TimeoutStream<S> {
	fn reset_timer(&mut self) {
		self.timeout
			.reset(std::time::Instant::now() + self.duration);
	}
}

impl<S: futures::Stream> futures::Stream for TimeoutStream<S> {
	type Error = TimeoutStreamError<S::Error>;
	type Item = S::Item;

	fn poll(&mut self) -> Result<Async<Option<Self::Item>>, Self::Error> {
		match self.stream.poll() {
			Ok(Async::Ready(None)) => Ok(Async::Ready(None)), // end of stream
			Ok(Async::Ready(item)) => {
				// not end of stream: reset timeout
				self.reset_timer();
				Ok(Async::Ready(item))
			},
			Ok(Async::NotReady) => {
				// check timeout
				match self.timeout.poll() {
					// timed out?
					Ok(Async::Ready(_)) => {
						// not an error
						Ok(Async::Ready(None))
						// Err(TimeoutStreamError::Timeout)
					},
					// still time left
					Ok(Async::NotReady) => Ok(Async::NotReady),
					Err(e) => Err(TimeoutStreamError::TimeoutError(e)),
				}
			},
			Err(e) => Err(TimeoutStreamError::StreamError(e)),
		}
	}
}
