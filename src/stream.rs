use futures::{
	self,
	sync::mpsc,
	Async,
};
use std::{
	io,
	os::raw::c_void,
};

use crate::error::Error;
use crate::evented::EventedDNSService;
use crate::ffi;
use crate::raw::DNSService;

#[allow(clippy::borrowed_box)]
fn box_raw<T>(ptr: &mut Box<T>) -> *mut c_void {
	ptr.as_mut() as *mut T as *mut c_void
}

type CallbackContext<T> = mpsc::UnboundedSender<io::Result<T>>;

#[must_use = "streams do nothing unless polled"]
pub(crate) struct ServiceStream<T> {
	service: EventedDNSService,
	_sender: Box<CallbackContext<T>>,
	receiver: mpsc::UnboundedReceiver<io::Result<T>>,
}

impl<T> ServiceStream<T> {
	pub(crate) fn run_callback<F>(
		context: *mut c_void,
		error_code: ffi::DNSServiceErrorType,
		f: F,
	) where
		F: FnOnce() -> io::Result<T>,
		T: ::std::fmt::Debug,
	{
		let sender = context as *mut CallbackContext<T>;
		let sender: &mut CallbackContext<T> = unsafe { &mut *sender };

		let data = Error::from(error_code)
			.map_err(io::Error::from)
			.and_then(|()| f());

		sender
			.unbounded_send(data)
			.expect("receiver must still be alive");
	}

	pub(crate) fn new<F>(f: F) -> io::Result<Self>
	where
		F: FnOnce(*mut c_void) -> Result<DNSService, Error>,
	{
		let (sender, receiver) = mpsc::unbounded::<io::Result<T>>();
		let mut sender = Box::new(sender);

		let service = f(box_raw(&mut sender))?;
		let service = EventedDNSService::new(service)?;

		Ok(ServiceStream {
			service,
			_sender: sender,
			receiver,
		})
	}
}

impl<T> futures::Stream for ServiceStream<T> {
	type Error = io::Error;
	type Item = T;

	fn poll(&mut self) -> Result<Async<Option<Self::Item>>, Self::Error> {
		self.service.poll()?;
		match self.receiver.poll() {
			Ok(Async::Ready(None)) => Ok(Async::Ready(None)),
			Ok(Async::Ready(Some(item))) => Ok(Async::Ready(Some(item?))),
			Ok(Async::NotReady) => Ok(Async::NotReady),
			Err(()) => unreachable!(),
		}
	}
}
