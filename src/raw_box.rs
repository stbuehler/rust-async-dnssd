pub struct RawBox<T>(*mut T);

impl<T> Drop for RawBox<T> {
	fn drop(&mut self) {
		// reconstruct Box data and let it drop
		unsafe { Box::from_raw(self.0) };
	}
}

impl<T> RawBox<T> {
	pub fn new(data: T) -> RawBox<T> {
		RawBox(Box::into_raw(Box::new(data)))
	}

	// pub fn new_from_box(data: Box<T>) -> RawBox<T> {
	// 	RawBox(Box::into_raw(data))
	// }

	pub fn get_ptr(&self) -> *mut T {
		self.0
	}
}
