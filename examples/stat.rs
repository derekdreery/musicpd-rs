// This example gets status info from mpd
extern crate musicpd;
extern crate futures;
extern crate tokio_core;


use tokio_core::reactor::Core;
use musicpd::client::{TokioMpc, TokioMpcNew, default_address};
use futures::Future;

fn main() {
    let addr = default_address();
    let mut core = Core::new().unwrap();
    let c: TokioMpcNew = TokioMpc::new(&addr, &core.handle());
    c.and_then(|client| {
        println!("test");
    });
    core.run(c).unwrap();
}

