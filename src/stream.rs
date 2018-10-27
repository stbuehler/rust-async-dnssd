use futures::sync::mpsc;
use futures::{self,Async};
use std::io;
use tokio_core::reactor::{Remote};

use evented::EventedDNSService;
use raw_box::RawBox;
use remote::GetRemote;

pub struct ServiceStream<T> {
	service: EventedDNSService,
	_sender: RawBox<mpsc::UnboundedSender<io::Result<T>>>,
	receiver: mpsc::UnboundedReceiver<io::Result<T>>,
}

impl<T> ServiceStream<T> {
	pub fn new<F>(f: F) -> io::Result<Self>
	where F: FnOnce(*mut mpsc::UnboundedSender<io::Result<T>>) -> io::Result<EventedDNSService>
	{
		let (sender, receiver) = mpsc::unbounded::<io::Result<T>>();
		let sender = RawBox::new(sender);

		let service = f(sender.get_ptr())?;

		Ok(ServiceStream{
			service,
			_sender: sender,
			receiver,
		})
	}
}

impl<T> futures::Stream for ServiceStream<T> {
	type Item = T;
	type Error = io::Error;

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

impl<T> GetRemote for ServiceStream<T> {
	fn remote(&self) -> &Remote {
		self.service.remote()
	}
}
