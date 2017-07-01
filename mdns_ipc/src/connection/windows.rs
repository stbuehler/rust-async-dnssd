use bytes::{Bytes};
use futures::{Future};
use futures::sync::mpsc;
use mdns_ipc_core::Deserialize;
use std::io;
use tokio_core::net::{TcpStream}; // ,TcpListener
use tokio_core::reactor::{Handle};
use tokio_io::io::{write_all};

use status::*;
use reader::*;
use enums::*;

pub type BIOFuture<T> = Box<Future<Item = T, Error = io::Error>>;

pub struct RawConnection {
	stream: TcpStream,
}
impl RawConnection {
	pub fn connect(handle: &Handle) -> BIOFuture<RawConnection> {
		use std::net;
		let address = net::SocketAddr::V4(net::SocketAddrV4::new(net::Ipv4Addr::new(127, 0, 0, 1), 5354));

		TcpStream::connect(&address, handle)
		.and_then(|stream| Ok(RawConnection{
			stream: stream,
		})).boxed()
	}

	// no async responses, won't share the connection
	pub fn send_short_request<StatusData>(self, request: Bytes) -> BIOFuture<(RawConnection,Status<StatusData>)>
	where StatusData: Deserialize+Send+'static
	{
		write_all(self.stream, request)
		.and_then(|(stream, _request)| {
			read_struct::<_, Status<StatusData>>(stream)
		})
		.and_then(|(stream, status)| {
			Ok((RawConnection{stream}, status))
		})
		.boxed()
	}
}

pub struct Response {

}

pub enum ExpectedResponse {
	None,
	Many(mpsc::UnboundedSender<Response>),
}

pub trait StatusDecoder {
	fn decode_status(&self, buffer: &mut Bytes) -> io::Result<ExpectedResponse>;
}

pub enum Request {
	CancelRequest{ctx: u64,},
	ShortRequest{flags: IpcFlags, op: Operation, ctx: u64, record: u32, decoder: &'static StatusDecoder,},
	LongRequest{flags: IpcFlags, op: Operation, ctx: u64, record: u32, decoder: &'static StatusDecoder,},
}
