extern crate async_dnssd;
extern crate futures;
extern crate tokio_core;

use std::env;
use async_dnssd::{browse, Interface};
use futures::{Stream, Future};
use tokio_core::reactor::Core;

fn main() {
    let mut core = Core::new().unwrap();
    let handle = core.handle();

    // MetaQuery to list all services available.
    let list_all_services = "_services._dns-sd._udp";

    // Use `cargo run --example browse` to list all services broadcasting
    // or `cargo run --example browse -- _http._tcp` to resolve a service.
    let query = env::args().nth(1).unwrap_or(list_all_services.to_string());
    println!("Query: {}", query);

    let browse_results =
        browse(Interface::Any, &query, None, &handle).ok().unwrap().for_each(move |service| {
            // Skip resolving for MetaQuery responses.
            if &query == list_all_services {
                println!("service: {}.{}", service.service_name, service.reg_type.split('.').next().unwrap());
                return Ok(())
            }

            println!("{:?}", service);
            match service.resolve(&handle) {
                    Ok(r) => {
                        handle.spawn(
                            r.for_each(|r| {
                            println!("interface: {:?}\n\
                                        fullname: {:?}\n\
                                        host_target: {:?}\n\
                                        port: {:?}\n\
                                        txt: {:?}\n",
                                    r.interface,
                                    r.fullname,
                                    r.host_target,
                                    r.port,
                                    String::from_utf8_lossy(&r.txt));
                                Ok(())
                            }).then(|_| Ok(()))
                        )
                    }
                    Err(e) => println!("Error resolving: {:?}", e),
            };

            Ok(())
        });

    core.run(browse_results).unwrap();

    // This will never be reached as `browse` will run forever in this example.
}
