use futures::sync::oneshot;
use futures::{self,Async};
use std::io;
use std::rc::Rc;
use tokio_core::reactor::{Remote};

use evented::EventedDNSService;
use raw::DNSService;
use raw_box::RawBox;
use remote::GetRemote;

struct Inner<T> {
	service: EventedDNSService,
	_sender: RawBox<oneshot::Sender<io::Result<T>>>,
	receiver: oneshot::Receiver<io::Result<T>>,
}

pub struct ServiceFuture<T>(Option<Inner<T>>);

impl<T> ServiceFuture<T> {
	pub fn new<F>(f: F) -> io::Result<Self>
	where F: FnOnce(*mut oneshot::Sender<io::Result<T>>) -> io::Result<EventedDNSService>
	{
		let (sender, receiver) = oneshot::channel::<io::Result<T>>();
		let sender = RawBox::new(sender);

		let service = f(sender.get_ptr())?;

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
	_sender: RawBox<oneshot::Sender<io::Result<T>>>,
	receiver: oneshot::Receiver<io::Result<T>>,
}

impl<T> ServiceFutureSingle<T> {
	pub fn new<R, F>(service: Rc<EventedDNSService>, f: F) -> io::Result<(Self, R)>
	where F: FnOnce(*mut oneshot::Sender<io::Result<T>>) -> io::Result<R>
	{
		let (sender, receiver) = oneshot::channel::<io::Result<T>>();
		let sender = RawBox::new(sender);

		let res = f(sender.get_ptr())?;

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
