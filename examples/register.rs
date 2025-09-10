use futures::StreamExt;

#[tokio::main(flavor = "current_thread")]
async fn main() -> std::io::Result<()> {
	// Use `cargo run --example register`

	let mut registration = async_dnssd::register("_ssh._tcp", 2022)?;
	let result = registration.next().await.expect("no stream end")?;
	println!("Registered: {:?}", result);

	// wait until killed
	registration.for_each(|_| async {}).await;
	Ok(())
}
