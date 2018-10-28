extern crate async_dnssd;
extern crate futures;
extern crate tokio_core;

use futures::stream::Stream;
use tokio_core::reactor::Core;

fn main() {
	// Use `cargo run --example enumerate-domains`

	let mut core = Core::new().unwrap();
	let handle = core.handle();

	let listing = async_dnssd::enumerate_domains(
		async_dnssd::Enumerate::BrowseDomains,
		async_dnssd::Interface::Any,
		&handle,
	)
	.unwrap()
	.for_each(|e| {
		println!("Domain: {:?}", e);
		Ok(())
	});
	core.run(listing).unwrap();
}
