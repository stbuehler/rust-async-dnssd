use std::{
	fmt,
	io,
	net::{
		IpAddr,
		Ipv4Addr,
		Ipv6Addr,
		SocketAddr,
		SocketAddrV4,
		SocketAddrV6,
	},
	pin::Pin,
	task::{
		Context,
		Poll,
	},
};

use futures::{
	prelude::*,
	stream,
	Stream,
};

use crate::{
	dns_consts::{
		Class,
		Type,
	},
	interface::Interface,
	service::{
		query_record_extended,
		QueryRecordData,
		QueryRecordFlags,
		QueryRecordResult,
	},
};

fn decode_a(a: QueryRecordResult) -> Option<(IpAddr, Interface)> {
	if a.rr_class == Class::IN && a.rr_type == Type::A && a.rdata.len() == 4 {
		let mut octets = [0u8; 4];
		octets.clone_from_slice(&a.rdata);
		Some((IpAddr::V4(Ipv4Addr::from(octets)), a.interface))
	} else {
		println!("Invalid A response: {:?}", a);
		None
	}
}

fn decode_aaaa(a: QueryRecordResult) -> Option<(IpAddr, Interface)> {
	if a.rr_class == Class::IN && a.rr_type == Type::AAAA && a.rdata.len() == 16 {
		let mut octets = [0u8; 16];
		octets.clone_from_slice(&a.rdata);
		Some((IpAddr::V6(Ipv6Addr::from(octets)), a.interface))
	} else {
		println!("Invalid AAAA response: {:?}", a);
		None
	}
}

/// Optional data when querying for a record; either use its default
/// value or customize it like:
///
/// ```
/// # use async_dnssd::ResolveHostData;
/// # use async_dnssd::QueryRecordFlags;
/// ResolveHostData {
///     flags: QueryRecordFlags::LONG_LIVED_QUERY,
///     ..Default::default()
/// };
/// ```
#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct ResolveHostData {
	/// flags for query
	pub flags: QueryRecordFlags,
	/// interface to query records on
	pub interface: Interface,
}

/// Pending resolve
#[must_use = "streams do nothing unless polled"]
pub struct ResolveHost {
	// inner: stream::BoxStream<'static, io::Result<(IpAddr, Interface)>>,
	inner: Pin<Box<dyn Stream<Item = io::Result<(IpAddr, Interface)>> + 'static + Send + Sync>>,
	port: u16,
}

impl Stream for ResolveHost {
	type Item = io::Result<ScopedSocketAddr>;

	fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
		Poll::Ready(
			futures::ready!(self.inner.poll_next_unpin(cx)?)
				.map(|(ip, iface)| Ok(ScopedSocketAddr::new(ip, self.port, iface.scope_id()))),
		)
	}
}

/// IP address with port and "scope id" (even for IPv4)
///
/// When converting to `SocketAddr` the "scope id" is lost for IPv4; when converting to
/// `SocketAddrV6` it uses `to_ipv6_mapped()` for IPv4 addresses.
pub enum ScopedSocketAddr {
	/// IPv4 target
	V4 {
		/// IP address
		address: Ipv4Addr,
		/// Port
		port: u16,
		/// Scope id (interface index; 0 for any)
		scope_id: u32,
	},
	/// IPv6 target
	V6 {
		/// IP address
		address: Ipv6Addr,
		/// Port
		port: u16,
		/// Scope id (interface index; 0 for any)
		scope_id: u32,
	},
}

impl ScopedSocketAddr {
	/// Create new `ScopedSocketAddr`
	pub fn new(address: IpAddr, port: u16, scope_id: u32) -> Self {
		match address {
			IpAddr::V4(address) => ScopedSocketAddr::V4 {
				address,
				port,
				scope_id,
			},
			IpAddr::V6(address) => ScopedSocketAddr::V6 {
				address,
				port,
				scope_id,
			},
		}
	}
}

impl Into<SocketAddr> for ScopedSocketAddr {
	fn into(self) -> SocketAddr {
		match self {
			ScopedSocketAddr::V4 { address, port, .. } => {
				// doesn't use scope_id
				SocketAddr::V4(SocketAddrV4::new(address, port))
			},
			ScopedSocketAddr::V6 {
				address,
				port,
				scope_id,
			} => SocketAddr::V6(SocketAddrV6::new(address, port, 0, scope_id)),
		}
	}
}

impl Into<SocketAddrV6> for ScopedSocketAddr {
	fn into(self) -> SocketAddrV6 {
		match self {
			ScopedSocketAddr::V4 {
				address,
				port,
				scope_id,
			} => SocketAddrV6::new(address.to_ipv6_mapped(), port, 0, scope_id),
			ScopedSocketAddr::V6 {
				address,
				port,
				scope_id,
			} => SocketAddrV6::new(address, port, 0, scope_id),
		}
	}
}

impl fmt::Display for ScopedSocketAddr {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			ScopedSocketAddr::V4 {
				address,
				port,
				scope_id: 0,
			} => write!(f, "{}:{}", address, port),
			ScopedSocketAddr::V4 {
				address,
				port,
				scope_id,
			} => write!(f, "[{}%{}]:{}", address, scope_id, port),
			ScopedSocketAddr::V6 {
				address,
				port,
				scope_id: 0,
			} => write!(f, "[{}]:{}", address, port),
			ScopedSocketAddr::V6 {
				address,
				port,
				scope_id,
			} => write!(f, "[{}%{}]:{}", address, scope_id, port),
		}
	}
}

impl fmt::Debug for ScopedSocketAddr {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		fmt::Display::fmt(self, f)
	}
}

/// Resolves hostname (with passed port) to stream of `ScopedSocketAddr`.
///
/// Uses
/// [`DNSServiceQueryRecord`](https://developer.apple.com/documentation/dnssd/1804747-dnsservicequeryrecord)
/// to query for `A` and `AAAA` records (in the `IN` class).
pub fn resolve_host_extended(
	host: &str,
	port: u16,
	data: ResolveHostData,
) -> io::Result<ResolveHost> {
	let qrdata = QueryRecordData {
		flags: data.flags,
		interface: data.interface,
		rr_class: Class::IN,
	};

	let inner_v6 = query_record_extended(host, Type::AAAA, qrdata)?
		.try_filter_map(|addr| async { Ok(decode_aaaa(addr)) });
	let inner_v4 = query_record_extended(host, Type::A, qrdata)?
		.try_filter_map(|addr| async { Ok(decode_a(addr)) });
	let inner = Box::pin(stream::select(inner_v6, inner_v4));

	Ok(ResolveHost { inner, port })
}
