use jdwp::commands::{
    reference_type::{ClassLoader, Signature},
    virtual_machine::ClassesBySignature,
};

mod common;

use common::Result;

const CASES: &[&str] = &["Ljava/lang/String;", "Ljava/util/List;", "[I"];

#[test]
fn signature() -> Result {
    let mut client = common::launch_and_attach("basic")?;

    for &case in CASES {
        let reply = client.send(ClassesBySignature::new(case))?;
        let id = reply[0].type_id;
        let signature = client.send(Signature::new(*id))?;
        assert_eq!(signature, case);
    }

    Ok(())
}

#[test]
fn class_loader() -> Result {
    let mut client = common::launch_and_attach("basic")?;

    let reply = client.send(ClassesBySignature::new(CASES[0]))?;
    let id = reply[0].type_id;

    // should be loaded with the system class loader
    assert!(matches!(client.send(ClassLoader::new(*id))?, None));

    Ok(())
}
