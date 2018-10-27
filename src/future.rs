use futures::sync::oneshot;
use futures::{self,Async};
use std::io;
use std::rc::Rc;
use std::os::raw::c_void;
use tokio_core::reactor::{Handle, Remote};

use error::Error;
use evented::EventedDNSService;
use ffi;
use raw::DNSService;
use raw_box::RawBox;
use remote::GetRemote;

type CallbackContext<T> = Option<oneshot::Sender<io::Result<T>>>;

struct Inner<T> {
	service: EventedDNSService,
	_sender: RawBox<CallbackContext<T>>,
	receiver: oneshot::Receiver<io::Result<T>>,
}

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

		let data = Error::from(error_code).map_err(io::Error::from).and_then(|()| f());

		sender.send(data).expect("receiver must still be alive");
	}

	pub fn new<F>(handle: &Handle, f: F) -> io::Result<Self>
	where F: FnOnce(*mut c_void) -> Result<DNSService, Error>
	{
		let (sender, receiver) = oneshot::channel::<io::Result<T>>();
		let sender = RawBox::new(Some(sender));

		let service = f(sender.get_ptr() as *mut c_void)?;
		let service = EventedDNSService::new(service, handle)?;

		Ok(ServiceFuture(Some(Inner{
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

	pub fn service(&self) -> &DNSService {
		&self.inner().service.service()
	}
}

impl<T> futures::Future for ServiceFuture<T> {
	type Item = (EventedDNSService, T);
	type Error = io::Error;

	fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
		if self.0.is_none() {
			// can only get ready once.
			return Ok(Async::NotReady);
		}
		self.inner_mut().service.poll()?;
		match self.inner_mut().receiver.poll() {
			Ok(Async::Ready(item)) => Ok(Async::Ready((
				self.0.take().unwrap().service,
				item?
			))),
			Ok(Async::NotReady) => Ok(Async::NotReady),
			Err(futures::Canceled) => unreachable!(),
		}
	}
}

impl<T> GetRemote for ServiceFuture<T> {
	fn remote(&self) -> &Remote {
		self.inner().service.remote()
	}
}

pub struct ServiceFutureSingle<T> {
	service: Rc<EventedDNSService>,
	_sender: RawBox<CallbackContext<T>>,
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

		let data = Error::from(error_code).map_err(io::Error::from).and_then(|()| f());

		sender.send(data).expect("receiver must still be alive");
	}

	pub fn new<R, F>(service: Rc<EventedDNSService>, f: F) -> io::Result<(Self, R)>
	where F: FnOnce(*mut c_void) -> Result<R, Error>
	{
		let (sender, receiver) = oneshot::channel::<io::Result<T>>();
		let sender = RawBox::new(Some(sender));

		let res = f(sender.get_ptr() as *mut c_void)?;

		Ok((ServiceFutureSingle{
			service,
			_sender: sender,
			receiver,
		}, res))
	}
}

impl<T> futures::Future for ServiceFutureSingle<T> {
	type Item = T;
	type Error = io::Error;

	fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
		self.service.poll()?;
		match self.receiver.poll() {
			Ok(Async::Ready(item)) => Ok(Async::Ready(item?)),
			Ok(Async::NotReady) => Ok(Async::NotReady),
			Err(futures::Canceled) => unreachable!(),
		}
	}
}

impl<T> GetRemote for ServiceFutureSingle<T> {
	fn remote(&self) -> &Remote {
		self.service.remote()
	}
}
