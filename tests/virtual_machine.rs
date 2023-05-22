use jdwp::{
    client::ClientError,
    commands::{string_reference::Value, virtual_machine::*},
};

mod common;

use common::Result;

const CASES: &[&str] = &[
    "Ljava/lang/String;",
    "Ljava/util/List;",
    "[I",
    "[Ljava/lang/String;",
];

#[test]
fn version() -> Result {
    let mut client = common::launch_and_attach("basic")?;
    let reply = client.send(Version)?;

    let version = match common::java_version() {
        8 => (1, 8),
        v => (v, 0),
    };

    assert_eq!((reply.version_major, reply.version_minor), version);

    Ok(())
}

#[test]
fn class_by_signature() -> Result {
    let mut client = common::launch_and_attach("basic")?;

    let classes: Result<_> = CASES.iter().try_fold(Vec::new(), move |mut acc, item| {
        acc.extend(client.send(ClassesBySignature::new(*item))?);
        Ok(acc)
    });
    let classes = classes?;

    assert_snapshot!(classes, @r###"
    [
        UnnamedClass {
            type_id: Class(
                [opaque_id],
            ),
            status: ClassStatus(
                VERIFIED | PREPARED | INITIALIZED,
            ),
        },
        UnnamedClass {
            type_id: Interface(
                [opaque_id],
            ),
            status: ClassStatus(
                VERIFIED | PREPARED | INITIALIZED,
            ),
        },
        UnnamedClass {
            type_id: Array(
                [opaque_id],
            ),
            status: ClassStatus(
                0x0,
            ),
        },
        UnnamedClass {
            type_id: Array(
                [opaque_id],
            ),
            status: ClassStatus(
                0x0,
            ),
        },
    ]
    "###);

    Ok(())
}

#[test]
fn all_classes() -> Result {
    let mut client = common::launch_and_attach("basic")?;

    let classes = client.send(AllClasses)?;
    let mut filtered = classes
        .iter()
        .filter_map(|c| CASES.contains(&&*c.signature).then_some(c))
        .collect::<Vec<_>>();
    filtered.sort_unstable_by_key(|c| c.signature.clone());

    // safe to assume that more classes that those are loaded by the JVM
    assert!(classes.len() > CASES.len());

    assert_snapshot!(filtered, @r###"
    [
        Class {
            type_id: Class(
                [opaque_id],
            ),
            signature: "Ljava/lang/String;",
            status: ClassStatus(
                VERIFIED | PREPARED | INITIALIZED,
            ),
        },
        Class {
            type_id: Interface(
                [opaque_id],
            ),
            signature: "Ljava/util/List;",
            status: ClassStatus(
                VERIFIED | PREPARED | INITIALIZED,
            ),
        },
        Class {
            type_id: Array(
                [opaque_id],
            ),
            signature: "[I",
            status: ClassStatus(
                0x0,
            ),
        },
        Class {
            type_id: Array(
                [opaque_id],
            ),
            signature: "[Ljava/lang/String;",
            status: ClassStatus(
                0x0,
            ),
        },
    ]
    "###);

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

    assert!(reply
        .classpaths
        .into_iter()
        .any(|cp| cp.starts_with("target/java")));

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
