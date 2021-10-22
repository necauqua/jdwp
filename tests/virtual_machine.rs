use jdwp::commands::virtual_machine::Version;

mod common;

#[test]
fn version() {
    let mut client = common::attach();

    let (_, reply) = client.send(Version).unwrap();

    let version = match &*std::env::var("JAVA_VERSION").unwrap() {
        "8" => (1, 8),
        v => (v.parse().unwrap(), 0),
    };
    assert_eq!((reply.version_major, reply.version_minor), version);
}
