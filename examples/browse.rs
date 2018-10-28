extern crate async_dnssd;
extern crate futures;
extern crate tokio_core;

use async_dnssd::{
	Class,
	TimeoutTrait,
	Type,
};
use futures::{
	Future,
	Stream,
};
use std::{
	env,
	net::{
		IpAddr,
		Ipv4Addr,
		Ipv6Addr,
	},
	time::Duration,
};
use tokio_core::reactor::Core;

fn decode_address(a: &async_dnssd::QueryRecordResult) -> Option<IpAddr> {
	if a.rr_class == Class::IN {
		if a.rr_type == Type::A && a.rdata.len() == 4 {
			let mut octets = [0u8; 4];
			octets.clone_from_slice(&a.rdata);
			Some(IpAddr::V4(Ipv4Addr::from(octets)))
		} else if a.rr_type == Type::AAAA && a.rdata.len() == 16 {
			let mut octets = [0u8; 16];
			octets.clone_from_slice(&a.rdata);
			Some(IpAddr::V6(Ipv6Addr::from(octets)))
		} else {
			None
		}
	} else {
		None
	}
}

fn main() {
	let mut core = Core::new().unwrap();
	let handle = core.handle();

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

	let listing =
		async_dnssd::browse(async_dnssd::Interface::Any, &query, None, &handle)
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
					let reg_type =
						service.service_name.clone() + "." + reg_type;
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

				let inner_handle = handle.clone();
				handle.spawn(
					match service.resolve(&handle) {
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
						println!(
							"Resolved {:?}: {:?}:{} (txt {:?})\t\t[{:?}]",
							service.service_name,
							r.host_target,
							r.port,
							String::from_utf8_lossy(&r.txt),
							r
						);
						let host_target = r.host_target.clone();
						let host_target_e = r.host_target.clone();
						inner_handle.spawn(
							// Query IPv4
							async_dnssd::query_record(
								&r.host_target,
								Type::A,
								Default::default(),
								&inner_handle,
							)?
							.timeout(address_timeout)?
							.select(
								// Query IPv6 and merge the results
								async_dnssd::query_record(
									&r.host_target,
									Type::AAAA,
									Default::default(),
									&inner_handle,
								)?
								.timeout(address_timeout)?,
							)
							.map_err(|e| e.into_io_error())
							.for_each(move |a| {
								match decode_address(&a) {
									Some(addr) => println!(
										"Address for {}: {}\t\t[{:?}]",
										host_target, addr, a
									),
									None => println!(
										"Address for {}: <unknown>\t\t[{:?}]",
										host_target, a
									),
								}
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
	core.run(listing).unwrap();
}
