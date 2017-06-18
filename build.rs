extern crate pkg_config;

fn get_target() -> String {
	std::env::var("TARGET").unwrap()
}
fn main() {
	if !get_target().contains("darwin") {
		pkg_config::find_library("avahi-compat-libdns_sd").unwrap();
	}
}
