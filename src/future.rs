use futures::{
	self,
	sync::oneshot,
	Async,
};
use std::{
	io,
	os::raw::c_void,
	rc::Rc,
};

use crate::{
	error::Error,
	evented::EventedDNSService,
	ffi,
	raw::DNSService,
};

#[allow(clippy::borrowed_box)]
fn box_raw<T>(ptr: &mut Box<T>) -> *mut c_void {
	ptr.as_mut() as *mut T as *mut c_void
}

type CallbackContext<T> = Option<oneshot::Sender<io::Result<T>>>;

struct Inner<T> {
	service: EventedDNSService,
	_sender: Box<CallbackContext<T>>,
	receiver: oneshot::Receiver<io::Result<T>>,
}

#[must_use = "futures do nothing unless polled"]
pub(crate) struct ServiceFuture<T>(Option<Inner<T>>);

impl<T> ServiceFuture<T> {
	pub(crate) fn run_callback<F>(context: *mut c_void, error_code: ffi::DNSServiceErrorType, f: F)
	where
		F: FnOnce() -> io::Result<T>,
		T: ::std::fmt::Debug,
	{
		let sender = context as *mut CallbackContext<T>;
		let sender: &mut CallbackContext<T> = unsafe { &mut *sender };
		let sender = sender.take().expect("callback must be run only once");

		let data = Error::from(error_code)
			.map_err(io::Error::from)
			.and_then(|()| f());

		sender.send(data).expect("receiver must still be alive");
	}

	pub(crate) fn new<F>(f: F) -> io::Result<Self>
	where
		F: FnOnce(*mut c_void) -> Result<DNSService, Error>,
	{
		let (sender, receiver) = oneshot::channel::<io::Result<T>>();
		let mut sender = Box::new(Some(sender));

		let service = f(box_raw(&mut sender))?;
		let service = EventedDNSService::new(service)?;

		Ok(ServiceFuture(Some(Inner {
			service,
			_sender: sender,
			receiver,
		})))
	}

	fn inner(&self) -> &Inner<T> {
		self.0.as_ref().expect("can only get ready once")
	}

	fn inner_mut(&mut self) -> &mut Inner<T> {
		self.0.as_mut().expect("can only get ready once")
	}

	pub(crate) fn service(&self) -> &DNSService {
		&self.inner().service.service()
	}
}

impl<T> futures::Future for ServiceFuture<T> {
	type Error = io::Error;
	type Item = (EventedDNSService, T);

	fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
		if self.0.is_none() {
			// can only get ready once.
			return Ok(Async::NotReady);
		}
		self.inner_mut().service.poll()?;
		match self.inner_mut().receiver.poll() {
			Ok(Async::Ready(item)) => Ok(Async::Ready((self.0.take().unwrap().service, item?))),
			Ok(Async::NotReady) => Ok(Async::NotReady),
			Err(futures::Canceled) => unreachable!(),
		}
	}
}

#[must_use = "futures do nothing unless polled"]
pub(crate) struct ServiceFutureSingle<T> {
	service: Rc<EventedDNSService>,
	_sender: Box<CallbackContext<T>>,
	receiver: oneshot::Receiver<io::Result<T>>,
}

impl<T> ServiceFutureSingle<T> {
	pub(crate) fn run_callback<F>(context: *mut c_void, error_code: ffi::DNSServiceErrorType, f: F)
	where
		F: FnOnce() -> io::Result<T>,
		T: ::std::fmt::Debug,
	{
		let sender = context as *mut CallbackContext<T>;
		let sender: &mut CallbackContext<T> = unsafe { &mut *sender };
		let sender = sender.take().expect("callback must be run only once");

		let data = Error::from(error_code)
			.map_err(io::Error::from)
			.and_then(|()| f());

		sender.send(data).expect("receiver must still be alive");
	}

	pub(crate) fn new<R, F>(service: Rc<EventedDNSService>, f: F) -> io::Result<(Self, R)>
	where
		F: FnOnce(*mut c_void) -> Result<R, Error>,
	{
		let (sender, receiver) = oneshot::channel::<io::Result<T>>();
		let mut sender = Box::new(Some(sender));

		let res = f(box_raw(&mut sender))?;

		Ok((
			ServiceFutureSingle {
				service,
				_sender: sender,
				receiver,
			},
			res,
		))
	}
}

impl<T> futures::Future for ServiceFutureSingle<T> {
	type Error = io::Error;
	type Item = T;

	fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
		self.service.poll()?;
		match self.receiver.poll() {
			Ok(Async::Ready(item)) => Ok(Async::Ready(item?)),
			Ok(Async::NotReady) => Ok(Async::NotReady),
			Err(futures::Canceled) => unreachable!(),
		}
	}
}
