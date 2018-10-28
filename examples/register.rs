extern crate async_dnssd;
extern crate futures;
extern crate tokio_core;

use async_dnssd::register;

fn main() -> std::io::Result<()> {
	// Use `cargo run --example register`

	let mut core = tokio_core::reactor::Core::new()?;
	let handle = core.handle();
	let (_registration, result) =
		core.run(register("_ssh._tcp", 2022, Default::default(), &handle)?)?;
	println!("Registered: {:?}", result);
	// wait until killed
	core.run(futures::future::empty::<(), ()>()).unwrap();
	Ok(())
}
