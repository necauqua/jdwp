use jdwp::client::JdwpClient;

pub fn attach() -> JdwpClient {
    JdwpClient::attach(("localhost", 1044)).expect("No JVM is runnning")
}
