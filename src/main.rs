use jdwp::client::JdwpClient;
use jdwp::commands::virtual_machine;
use std::error::Error;

// this is some CRAZY TESTING fyi
pub fn main() -> Result<(), Box<dyn Error>> {
    let mut client = JdwpClient::attach(("localhost", 1044))?;

    let (_, data) = client.send(virtual_machine::Version)?;
    println!("{:#?}", data);

    let (_, data) = client.send(virtual_machine::Exit::new(0))?;
    println!("{:#?}", data);

    Ok(())
}
