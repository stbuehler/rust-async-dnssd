extern crate async_dnssd;
extern crate futures;
extern crate tokio;

use async_dnssd::register;

fn main() -> std::io::Result<()> {
	// Use `cargo run --example register`
	let mut rt = tokio::runtime::current_thread::Runtime::new()?;

	let (_registration, result) =
		rt.block_on(register("_ssh._tcp", 2022)?)?;
	println!("Registered: {:?}", result);

	// wait until killed
	rt.block_on(futures::future::empty::<(), ()>()).unwrap();
	Ok(())
}
