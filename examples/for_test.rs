extern crate musicpd;

use musicpd::client::Client;
use musicpd::protocol::command::{Command, CommandList};
use musicpd::types::TagType;

fn main() {
    println!("This example will change often, it is for me to test");

    let mut c = Client::connect("127.0.0.1:6600").unwrap();
    let mut list = CommandList::new();
    list.push(Command::Stats);
    //list.push(Command::Next);
    list.push(Command::Count {
        tag: (TagType::Genre, "Rock".into()),
        group: None
    });
    println!("{:?}", c.run_commands(list));
}
