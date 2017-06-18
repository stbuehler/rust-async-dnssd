use tokio_core::reactor::Remote;

pub trait GetRemote {
	fn remote(&self) -> &Remote;
}
