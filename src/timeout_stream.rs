use futures::prelude::*;
use std::{
	io,
	pin::Pin,
	task::{
		Context,
		Poll,
	},
	time::Duration,
};

/// `futures::Stream` extension to simplify building
/// [`TimeoutStream`](struct.TimeoutStream.html)
pub trait StreamTimeoutExt: Stream + Sized {
	/// Create new [`TimeoutStream`](struct.TimeoutStream.html)
	fn timeout(self, duration: Duration) -> io::Result<TimeoutStream<Self>>;
}

impl<S: Stream> StreamTimeoutExt for S {
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
	timeout: tokio::time::Delay,
}

impl<S: Stream> TimeoutStream<S> {
	pin_utils::unsafe_pinned!(stream: S);

	pin_utils::unsafe_unpinned!(timeout: tokio::time::Delay);

	/// Create new `TimeoutStream`.
	///
	/// Also see [`StreamTimeoutExt::timeout`](trait.StreamTimeoutExt.html#method.timeout).
	pub fn new(stream: S, duration: Duration) -> io::Result<Self> {
		Ok(TimeoutStream {
			stream,
			duration,
			timeout: tokio::time::delay_for(duration),
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
	TimeoutError(tokio::time::Error),
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

impl<S: Stream> TimeoutStream<S> {
	fn reset_timer(self: Pin<&mut Self>) {
		let next = tokio::time::Instant::now() + self.duration;
		self.timeout().reset(next);
	}
}

impl<S: TryStream> Stream for TimeoutStream<S> {
	type Item = Result<S::Ok, S::Error>;

	fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
		match self.as_mut().stream().try_poll_next(cx) {
			Poll::Ready(None) => Poll::Ready(None), // end of stream
			Poll::Ready(Some(Ok(item))) => {
				// not end of stream: reset timeout
				self.reset_timer();
				Poll::Ready(Some(Ok(item)))
			},
			Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(e))),
			Poll::Pending => {
				// check timeout
				match self.timeout().poll_unpin(cx) {
					// timed out?
					Poll::Ready(()) => {
						// not an error
						Poll::Ready(None)
						// Err(TimeoutStreamError::Timeout)
					},
					// still time left
					Poll::Pending => Poll::Pending,
				}
			},
		}
	}
}
