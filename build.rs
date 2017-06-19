extern crate pkg_config;

use std::env::var_os;

fn cfg_family(family: &str) -> bool {
	var_os("CARGO_CFG_TARGET_FAMILY").unwrap() == *family
}

fn cfg_os(family: &str) -> bool {
	var_os("CARGO_CFG_TARGET_OS").unwrap() == *family
}

fn find_avahi_compat_dns_sd() {
	// on unix but not darwin link avahi compat
	if cfg_family("unix")
	&& !(cfg_os("macos") || cfg_os("ios")) {
		pkg_config::probe_library("avahi-compat-libdns_sd").unwrap();
	}
}

fn main() {
	find_avahi_compat_dns_sd();
}
