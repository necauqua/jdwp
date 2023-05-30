use std::assert_eq;

use jdwp::{
    client::ClientError,
    commands::{string_reference::Value, thread_reference, virtual_machine::*},
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

    let classes = CASES
        .iter()
        .map(|&signature| Ok(client.send(ClassBySignature::new(signature))?.0))
        .collect::<Result<Vec<_>>>()?;

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

    let mut thread_names = client
        .send(AllThreads)?
        .iter()
        .map(|id| Ok(client.send(thread_reference::Name::new(*id))?))
        .collect::<Result<Vec<_>>>()?;

    thread_names.sort_unstable();

    // there are at least those present
    let expected = &[
        "main",
        "Signal Dispatcher",
        "Reference Handler",
        "Finalizer",
    ];
    assert!(thread_names.len() >= expected.len());
    assert!(thread_names.iter().any(|n| expected.contains(&n.as_str())));

    Ok(())
}

#[test]
fn dispose() -> Result {
    let mut client = common::launch_and_attach("basic")?;

    client.send(Dispose)?;

    assert!(matches!(client.send(Version), Err(ClientError::Disposed)));

    Ok(())
}

#[test]
fn id_sizes() -> Result {
    let mut client = common::launch_and_attach("basic")?;

    let id_sizes = client.send(IDSizes)?;

    // Everything seems to just be 64bit everywhere I test it
    assert_snapshot!(id_sizes, @r###"
    IDSizeInfo {
        field_id_size: 8,
        method_id_size: 8,
        object_id_size: 8,
        reference_type_id_size: 8,
        frame_id_size: 8,
    }
    "###);

    Ok(())
}

#[test]
fn suspend_resume() -> Result {
    let mut client = common::launch_and_attach("basic")?;

    client.send(Suspend)?;
    client.send(Suspend)?;
    client.send(Resume)?;
    client.send(Resume)?;

    // extra resume should be a no-op
    client.send(Resume)?;

    Ok(())
}

#[test]
fn exit() -> Result {
    let mut client = common::launch_and_attach("basic")?;

    client.send(Exit::new(2))?;

    assert_eq!(client.jvm_process.wait()?.code(), Some(2));

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

#[test]
fn capabilities() -> Result {
    let mut client = common::launch_and_attach("basic")?;

    let capabilities = client.send(Capabilities)?;

    // those seem to be all enabled on JDKs I test with
    assert_snapshot!(capabilities, @r###"
    CapabilitiesReply {
        can_watch_field_modification: true,
        can_watch_field_access: true,
        can_get_bytecodes: true,
        can_get_synthetic_attribute: true,
        can_get_owned_monitor_info: true,
        can_get_current_contended_monitor: true,
        can_get_monitor_info: true,
    }
    "###);

    Ok(())
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
fn hold_release_events() -> Result {
    let mut client = common::launch_and_attach("basic")?;

    client.send(HoldEvents)?;
    client.send(HoldEvents)?;
    client.send(ReleaseEvents)?;
    client.send(ReleaseEvents)?;

    // extra release should be a no-op
    client.send(ReleaseEvents)?;

    Ok(())
}

#[test]
fn capabilities_new() -> Result {
    let mut client = common::launch_and_attach("basic")?;

    let capabilities = client.send(CapabilitiesNew)?;

    // on JDKs I test with this seems to be the case
    assert_snapshot!(capabilities, @r###"
    CapabilitiesNewReply {
        capabilities: CapabilitiesReply {
            can_watch_field_modification: true,
            can_watch_field_access: true,
            can_get_bytecodes: true,
            can_get_synthetic_attribute: true,
            can_get_owned_monitor_info: true,
            can_get_current_contended_monitor: true,
            can_get_monitor_info: true,
        },
        can_redefine_classes: true,
        can_add_method: false,
        can_unrestrictedly_redefine_classes: false,
        can_pop_frames: true,
        can_use_instance_filters: true,
        can_get_source_debug_extension: true,
        can_request_vmdeath_event: true,
        can_set_default_stratum: true,
        can_get_instance_info: true,
        can_request_monitor_events: true,
        can_get_monitor_frame_info: true,
        can_use_source_name_filters: false,
        can_get_constant_pool: true,
        can_force_early_return: true,
    }
    "###);

    Ok(())
}

#[test]
fn set_default_stratum() -> Result {
    let mut client = common::launch_and_attach("basic")?;

    // atm we have nothing to test so we just check that it doesn't error
    client.send(SetDefaultStratum::new("Not Java"))?;

    Ok(())
}

#[test]
fn instance_counts() -> Result {
    let mut client = common::launch_and_attach("basic")?;

    let id = client.send(ClassBySignature::new("LBasic;"))?.type_id;
    let id2 = client
        .send(ClassBySignature::new("LBasic$NestedClass;"))?
        .type_id;

    let counts = client.send(InstanceCounts::new(vec![*id, *id2]))?;

    assert_eq!(counts, [2, 0]);

    Ok(())
}
