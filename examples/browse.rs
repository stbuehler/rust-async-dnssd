use async_dnssd::{
	StreamTimeoutExt,
	TxtRecord,
};
use futures::{
	Future,
	Stream,
};
use std::{
	env,
	time::Duration,
};

fn main() {
	let mut rt = tokio::runtime::current_thread::Runtime::new().unwrap();

	let search_timeout = Duration::from_secs(10);
	let resolve_timeout = Duration::from_secs(3);
	let address_timeout = Duration::from_secs(3);

	// MetaQuery to list all services available.
	let list_all_services = "_services._dns-sd._udp";

	// Use `cargo run --example browse` to list all services broadcasting
	// or `cargo run --example browse -- _http._tcp` to resolve a service.
	let query = env::args()
		.nth(1)
		.unwrap_or_else(|| list_all_services.to_string());
	println!("Browse: {}", query);

	let listing = async_dnssd::browse(&query)
		.expect("failed browse")
		.timeout(search_timeout)
		.expect("failed timeout")
		.map_err(|e| e.into_io_error())
		.for_each(|service| {
			let added = service.flags.contains(async_dnssd::BrowsedFlags::ADD);
			if query == list_all_services {
				// resolving MetaQuery responses isn't useful (and fails
				// with "bad param")... we'd need to browse them

				let mut reg_type = service.reg_type.as_str();
				if reg_type.ends_with('.') {
					reg_type = &reg_type[..reg_type.len() - 1];
				}
				let reg_type = service.service_name.clone() + "." + reg_type;
				println!(
					"Service Type {}{:?}@{:?}\t\t[{:?}]",
					if added { '+' } else { '-' },
					reg_type,
					service.domain,
					service
				);

				return Ok(());
			}

			println!(
				"Service {}{:?}@{:?} (type {:?})\t\t[{:?}]",
				if added { '+' } else { '-' },
				service.service_name,
				service.domain,
				service.reg_type,
				service
			);

			if !added {
				// only resolve added services
				return Ok(());
			}

			let service_name_e = service.service_name.clone();

			tokio::runtime::current_thread::spawn(
				match service.resolve() {
					Ok(r) => r,
					Err(e) => {
						println!("resolve failed: {:?}", e);
						return Ok(());
					},
				}
				.timeout(resolve_timeout)
				.expect("failed timeout")
				.map_err(|e| e.into_io_error())
				.for_each(move |r| {
					let txt = TxtRecord::parse(&r.txt).map(|rdata|
						rdata
						.iter()
						.map(|(key, value)| (
							String::from(String::from_utf8_lossy(key)),
							value.map(|value| String::from(String::from_utf8_lossy(value))),
						))
						.collect::<Vec<_>>()
					);
					println!(
						"Resolved {:?} on {:?}: {:?}:{} (txt {:?})\t\t[{:?}]",
						service.service_name,
						r.interface,
						r.host_target,
						r.port,
						txt,
						r
					);
					let fullname = r.fullname.clone();
					let host_target_e = r.host_target.clone();
					tokio::runtime::current_thread::spawn(
						// Query IPv4 + IPv6
						r.resolve_socket_address()?
						.timeout(address_timeout)?
						.map_err(|e| e.into_io_error())
						.for_each(move |addr| {
							println!(
								"Address for {}: {}",
								fullname, addr
							);
							Ok(())
						})
						.or_else(move |e| {
							println!(
								"query_record {} failed: {}",
								host_target_e, e
							);
							Ok(())
						}),
					);
					Ok(())
				})
				.or_else(move |e| {
					println!("resolve {:?} failed: {}", service_name_e, e);
					Ok(())
				}),
			);

			Ok(())
		});
	rt.block_on(listing).unwrap();
	rt.run().unwrap();
}
