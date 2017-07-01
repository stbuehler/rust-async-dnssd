use bytes::{BytesMut,BufMut,Bytes};
use futures::{Async,Future};
use mdns_ipc_core::{Deserialize,SizeHint,parse};
use mdns_ipc_core::errors::is_need_more_bytes;
use std::io;
use std::marker::PhantomData;
use tokio_io::AsyncRead;
use tokio_io::io::{read_exact,ReadExact};


struct WindowedBuf {
	storage: BytesMut,
	have: usize,
}
impl WindowedBuf {
	fn new(want: usize) -> Self {
		let mut storage = BytesMut::new();
		storage.reserve(want);
		unsafe { storage.advance_mut(want); }
		WindowedBuf {
			storage: storage,
			have: 0,
		}
	}

	fn cont(mut storage: BytesMut, add: usize) -> Self {
		let have = storage.len();
		storage.reserve(add);
		unsafe { storage.advance_mut(add); }
		WindowedBuf {
			storage: storage,
			have: have,
		}
	}
}
impl AsMut<[u8]> for WindowedBuf {
	fn as_mut(&mut self) -> &mut [u8] {
		&mut self.storage[self.have..]
	}
}

pub struct ReadStruct<R: AsyncRead, S: Deserialize> {
	future: ReadExact<R, WindowedBuf>,
	marker: PhantomData<S>,
	exact: bool,
}
impl<R: AsyncRead, S: Deserialize> Future for ReadStruct<R, S> {
	type Item = (R, S);
	type Error = io::Error;

	fn poll(&mut self) -> io::Result<Async<Self::Item>> {
		Ok(match self.future.poll()? {
			Async::Ready((stream, buf)) => {
				if self.exact {
					Async::Ready((stream, parse(buf.storage.into())?))
				} else {
					let buf = Bytes::from(buf.storage);
					match parse(buf.clone()) {
						Err(e) => if let Some(add) = is_need_more_bytes(&e) {
							let buf = WindowedBuf::cont(buf.into(), add);
							self.future = read_exact(stream, buf);
							Async::NotReady
						} else {
							return Err(e);
						},
						Ok(result) => Async::Ready((stream, result)),
					}
				}
			},
			Async::NotReady => Async::NotReady,
		})
	}
}

pub fn read_struct<R: AsyncRead, S: Deserialize>(stream: R) -> ReadStruct<R, S> {
	let (need, exact) = match S::deserialize_size_hint() {
		SizeHint::Exact(n) => (n, true),
		SizeHint::AtLeast(n) => (n, false),
	};
	let buf = WindowedBuf::new(need);
	ReadStruct {
		future: read_exact(stream, buf),
		marker: PhantomData,
		exact: exact,
	}
}
