use std::process::Command;

use jdwp::{
    client::ClientError,
    commands::{string_reference::Value, virtual_machine::*},
    enums::{ClassStatus, TypeTag},
    types::TaggedReferenceTypeID,
};

mod common;

use common::Result;

#[test]
fn version() -> Result {
    let mut output = Command::new("javac").arg("-version").output()?;
    output.stderr.extend(output.stdout); // stderr hacks for java 8

    let version = String::from_utf8_lossy(&output.stderr)
        .lines()
        .last() // last line for java 8 as well, I personally have the _JAVA_OPTIONS cluttering stderr
        .unwrap()
        .chars()
        .skip(6) // 'javac '
        .take_while(|ch| ch.is_numeric())
        .collect::<String>()
        .parse()?;

    let version = match version {
        1 => (1, 8),
        v => (v, 0),
    };

    let mut client = common::launch_and_attach("basic")?;
    let reply = client.send(Version)?;
    assert_eq!((reply.version_major, reply.version_minor), version);

    Ok(())
}

mod class_by_signature {
    use super::*;

    #[test]
    fn class() -> Result {
        let mut client = common::launch_and_attach("basic")?;

        let class = &client.send(ClassesBySignature::new("Ljava/lang/String;"))?[0];

        assert!(matches!(class.type_id, TaggedReferenceTypeID::Class(_)));
        assert_eq!(class.status, ClassStatus::OK);

        Ok(())
    }

    #[test]
    fn interface() -> Result {
        let mut client = common::launch_and_attach("basic")?;

        let class = &client.send(ClassesBySignature::new("Ljava/util/List;"))?[0];

        assert!(matches!(class.type_id, TaggedReferenceTypeID::Interface(_)));
        assert_eq!(class.status, ClassStatus::OK);

        Ok(())
    }

    #[test]
    fn primitive_array() -> Result {
        let mut client = common::launch_and_attach("basic")?;

        let class = &client.send(ClassesBySignature::new("[I"))?[0];

        assert!(matches!(class.type_id, TaggedReferenceTypeID::Array(_)));
        assert_eq!(class.status, ClassStatus::empty());

        Ok(())
    }

    #[test]
    fn object_array() -> Result {
        let mut client = common::launch_and_attach("basic")?;

        let classes = client.send(ClassesBySignature::new("[Ljava/lang/String;"))?;

        assert_eq!(classes.len(), 1);
        let class = &classes[0];

        assert!(matches!(class.type_id, TaggedReferenceTypeID::Array(_)));
        assert_eq!(class.status, ClassStatus::empty());

        Ok(())
    }
}

#[test]
fn all_classes() -> Result {
    let mut client = common::launch_and_attach("basic")?;

    let mut test = 0;
    let mut string = 0;
    let mut list = 0;
    let mut array = 0;

    for class in client.send(AllClasses)? {
        if class.signature == "LBasic;" && class.type_id.tag() == TypeTag::Class {
            test += 1;
        }
        if class.signature == "Ljava/lang/String;" && class.type_id.tag() == TypeTag::Class {
            string += 1;
        }
        if class.signature == "Ljava/util/List;" && class.type_id.tag() == TypeTag::Interface {
            list += 1;
        }
        if class.signature == "[I" && class.type_id.tag() == TypeTag::Array {
            array += 1;
        }
    }

    assert_eq!(test, 1);
    assert_eq!(string, 1);
    assert_eq!(list, 1);
    assert_eq!(array, 1);

    Ok(())
}

#[test]
fn all_threads() -> Result {
    let mut client = common::launch_and_attach("basic")?;

    let thread_ids = client.send(AllThreads)?;

    // sanity check?.
    assert!(!thread_ids.is_empty());

    Ok(())
}

#[test]
fn top_level_thread_groups() -> Result {
    let mut client = common::launch_and_attach("basic")?;

    let thread_group_ids = client.send(TopLevelThreadGroups)?;

    assert!(!thread_group_ids.is_empty());

    Ok(())
}

#[test]
fn dispose() -> Result {
    let mut client = common::launch_and_attach("basic")?;

    client.send(Dispose)?;

    match client.send(Version) {
        Err(ClientError::Disposed) => Ok(()),
        res => panic!("expected a Disposed error, got: {:?}", res),
    }
}

#[test]
fn class_path() -> Result {
    let mut client = common::launch_and_attach("basic")?;

    let reply = client.send(ClassPaths)?;

    assert!(reply.classpaths.contains(&"target/java".to_owned()));

    Ok(())
}

#[test]
fn string_roundtrip() -> Result {
    let mut client = common::launch_and_attach("basic")?;

    let string = "this is a string";

    let string_id = client.send(CreateString::new(string))?;

    let string_value = client.send(Value::new(*string_id))?;

    assert_eq!(string_value, string);

    Ok(())
}
