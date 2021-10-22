use jdwp::commands::virtual_machine::Version;

mod common;

#[test]
fn version() {
    let mut client = common::attach();

    let (_, version) = client.send(Version).unwrap();

    assert_eq!(
        version.version_major,
        std::env::var("JAVA_VERSION").unwrap().parse().unwrap()
    );
}
