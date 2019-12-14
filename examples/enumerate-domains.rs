use futures::stream::Stream;

fn main() {
	// Use `cargo run --example enumerate-domains`

	let listing = async_dnssd::enumerate_domains(
		async_dnssd::Enumerate::BrowseDomains,
		async_dnssd::Interface::Any,
	)
	.unwrap()
	.for_each(|e| {
		println!("Domain: {:?}", e);
		Ok(())
	});
	tokio::runtime::current_thread::block_on_all(listing).unwrap();
}
