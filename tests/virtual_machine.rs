use std::assert_eq;

use jdwp::{
    highlevel::JvmObject,
    spec::{reference_type::InstanceLimit, virtual_machine::*},
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
    let vm = common::launch_and_attach_vm("basic")?;
    let reply = vm.version()?;

    let version = match common::java_version() {
        8 => (1, 8),
        v => (v, 0),
    };

    assert_eq!((reply.version_major, reply.version_minor), version);

    Ok(())
}

#[test]
fn class_by_signature() -> Result {
    let vm = common::launch_and_attach_vm("basic")?;

    let classes = CASES
        .iter()
        .map(|&signature| Ok(vm.class_by_signature(signature)?))
        .collect::<Result<Vec<_>>>()?;

    assert_snapshot!(classes, @r###"
    [
        (
            Class(
                WrapperJvmObject(
                    ClassID(opaque),
                ),
            ),
            ClassStatus(
                VERIFIED | PREPARED | INITIALIZED,
            ),
        ),
        (
            Interface(
                WrapperJvmObject(
                    InterfaceID(opaque),
                ),
            ),
            ClassStatus(
                VERIFIED | PREPARED | INITIALIZED,
            ),
        ),
        (
            Array(
                WrapperJvmObject(
                    ArrayTypeID(opaque),
                ),
            ),
            ClassStatus(
                0x0,
            ),
        ),
        (
            Array(
                WrapperJvmObject(
                    ArrayTypeID(opaque),
                ),
            ),
            ClassStatus(
                0x0,
            ),
        ),
    ]
    "###);

    Ok(())
}

#[test]
fn all_classes() -> Result {
    let vm = common::launch_and_attach_vm("basic")?;

    let classes = vm.all_classes()?;
    let mut filtered = classes
        .iter()
        .filter_map(|c| CASES.contains(&&*c.signature).then_some(c))
        .collect::<Vec<_>>();
    filtered.sort_unstable_by_key(|c| &c.signature);

    // safe to assume that more classes that those are loaded by the JVM
    assert!(classes.len() > CASES.len());

    assert_snapshot!(filtered, @r###"
    [
        Class {
            object: Class(
                WrapperJvmObject(
                    ClassID(opaque),
                ),
            ),
            signature: "Ljava/lang/String;",
            generic_signature: None,
            status: ClassStatus(
                VERIFIED | PREPARED | INITIALIZED,
            ),
        },
        Class {
            object: Interface(
                WrapperJvmObject(
                    InterfaceID(opaque),
                ),
            ),
            signature: "Ljava/util/List;",
            generic_signature: None,
            status: ClassStatus(
                VERIFIED | PREPARED | INITIALIZED,
            ),
        },
        Class {
            object: Array(
                WrapperJvmObject(
                    ArrayTypeID(opaque),
                ),
            ),
            signature: "[I",
            generic_signature: None,
            status: ClassStatus(
                0x0,
            ),
        },
        Class {
            object: Array(
                WrapperJvmObject(
                    ArrayTypeID(opaque),
                ),
            ),
            signature: "[Ljava/lang/String;",
            generic_signature: None,
            status: ClassStatus(
                0x0,
            ),
        },
    ]
    "###);

    Ok(())
}

#[test]
fn all_classes_generic() -> Result {
    let vm = common::launch_and_attach_vm("basic")?;

    const CASES: &[&str] = &[
        // "Ljava/lang/String;", has extra interfaces on jvm 17
        "Ljava/util/List;",
        "[I",
        "[Ljava/lang/String;",
    ];

    let classes = vm.all_classes_generic()?;
    let mut filtered = classes
        .iter()
        .filter_map(|c| CASES.contains(&&*c.signature).then_some(c))
        .collect::<Vec<_>>();
    filtered.sort_unstable_by_key(|c| &c.signature);

    assert!(classes.len() > CASES.len());

    assert_snapshot!(filtered, @r###"
    [
        Class {
            object: Interface(
                WrapperJvmObject(
                    InterfaceID(opaque),
                ),
            ),
            signature: "Ljava/util/List;",
            generic_signature: Some(
                "<E:Ljava/lang/Object;>Ljava/lang/Object;Ljava/util/Collection<TE;>;",
            ),
            status: ClassStatus(
                VERIFIED | PREPARED | INITIALIZED,
            ),
        },
        Class {
            object: Array(
                WrapperJvmObject(
                    ArrayTypeID(opaque),
                ),
            ),
            signature: "[I",
            generic_signature: None,
            status: ClassStatus(
                0x0,
            ),
        },
        Class {
            object: Array(
                WrapperJvmObject(
                    ArrayTypeID(opaque),
                ),
            ),
            signature: "[Ljava/lang/String;",
            generic_signature: None,
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
    let vm = common::launch_and_attach_vm("basic")?;

    let mut thread_names = vm
        .all_threads()?
        .iter()
        .map(|thread| Ok(thread.name()?))
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
    let mut vm = common::launch_and_attach_vm("basic")?;

    // just a smoke test I guess
    vm.take().dispose()?;

    Ok(())
}

#[test]
fn dispose_error() -> Result {
    let mut client = common::launch_and_attach("basic")?;

    client.send(Dispose)?;

    assert_snapshot!((client.send(Version), client.send(Version)), @r###"
    (
        Err(
            Disposed,
        ),
        Err(
            Disposed,
        ),
    )
    "###);

    Ok(())
}

#[test]
fn id_sizes() -> Result {
    let vm = common::launch_and_attach_vm("basic")?;

    let id_sizes = vm.id_sizes()?;

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
    let vm = common::launch_and_attach_vm("basic")?;

    // another smoke test/just coverage
    vm.suspend()?;
    vm.suspend()?;
    vm.resume()?;
    vm.resume()?;

    // extra resume should be a no-op
    vm.resume()?;

    Ok(())
}

#[test]
fn exit() -> Result {
    let mut vm = common::launch_and_attach_vm("basic")?;

    vm.take().exit(2)?;

    assert_eq!(vm.jvm_process.wait()?.code(), Some(2));

    Ok(())
}

#[test]
fn string_roundtrip() -> Result {
    let vm = common::launch_and_attach_vm("basic")?;

    let string = "this is a string with a secret: M日\u{10401}\u{7F}";

    let jvm_string = vm.create_string(string)?;
    let string_value = jvm_string.value()?;

    assert_eq!(string_value, string);

    Ok(())
}

#[test]
fn capabilities() -> Result {
    let vm = common::launch_and_attach_vm("basic")?;

    let capabilities = vm.capabilities()?;

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
    let vm = common::launch_and_attach_vm("basic")?;

    let reply = vm.classpaths()?;

    assert!(reply
        .classpaths
        .into_iter()
        .any(|cp| cp.starts_with("target/java")));

    Ok(())
}

#[test]
fn hold_release_events() -> Result {
    let vm = common::launch_and_attach_vm("basic")?;

    vm.hold_events()?;
    vm.hold_events()?;
    vm.release_events()?;
    vm.release_events()?;

    // extra release should be a no-op
    vm.release_events()?;

    Ok(())
}

#[test]
fn capabilities_new() -> Result {
    let vm = common::launch_and_attach_vm("basic")?;

    let capabilities = vm.capabilities_new()?;

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
    let vm = common::launch_and_attach_vm("basic")?;

    // another smoke, test if this doesn't just error
    vm.set_default_stratum("NotJava")?;

    Ok(())
}

#[test]
fn instance_counts() -> Result {
    let vm = common::launch_and_attach_vm("basic")?;

    let (ref_type, _) = vm.class_by_signature("LBasic;")?;
    let (ref_type2, _) = vm.class_by_signature("LBasic$NestedClass;")?;

    let counts = vm.instance_counts(vec![ref_type.id(), ref_type2.id()])?;
    let counts2 = [ref_type.instance_count()?, ref_type2.instance_count()?];

    assert_eq!(counts, [2, 0]);
    assert_eq!(counts2, [2, 0]);

    Ok(())
}

#[test]
fn dispose_object() -> Result {
    let vm = common::launch_and_attach_vm("basic")?;

    let (ref_type, _) = vm.class_by_signature("LBasic;")?;

    let instance = ref_type
        .instances(InstanceLimit::limit(1))?
        .into_iter()
        .next()
        .unwrap();

    // just a smoke too
    instance.dispose_single()?;

    Ok(())
}
