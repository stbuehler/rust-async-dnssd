use tokio_core::reactor::Remote;

/// Access `Remote` handle of `Future`s and `Stream`s supporting it
pub trait GetRemote {
	/// get `Remote` reference
	fn remote(&self) -> &Remote;
}
