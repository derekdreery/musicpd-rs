// This example gets status info from mpd
extern crate musicpd;

use musicpd::client::Client;
use musicpd::protocol::command;


fn main() {
    let mut c = Client::connect("127.0.0.1:6600").unwrap();
    println!("{:?}", c.version());
    let mut list = command::CommandList::new();
    list.push(command::Command::Status);
    println!("{:?}", c.run_commands(list));
}
