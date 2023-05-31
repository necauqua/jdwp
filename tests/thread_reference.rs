use std::assert_eq;

use common::Result;
use jdwp::{
    client::JdwpClient,
    commands::{
        reference_type::{Methods, Signature},
        thread_group_reference,
        thread_reference::{
            CurrentContendedMonitor, ForceEarlyReturn, FrameCount, FrameLimit, Frames, Name,
            OwnedMonitors, OwnedMonitorsStackDepthInfo, Resume, Status, Suspend, SuspendCount,
            ThreadGroup,
        },
        virtual_machine::AllThreads,
    },
    types::{TaggedReferenceTypeID, ThreadID, Value},
};

mod common;

fn get_main_thread(client: &mut JdwpClient) -> Result<ThreadID> {
    Ok(client
        .send(AllThreads)?
        .into_iter()
        .map(|id| Ok((id, client.send(Name::new(id))?)))
        .collect::<Result<Vec<_>>>()?
        .iter()
        .find(|(_, name)| name == "main")
        .expect("Didn't find main thread")
        .0)
}

// a macro so that same assert_snapshot is not called from multiple places as
// insta seems to hate that
macro_rules! check_host_suspended {
    ($client:expr, $thread:expr, $command:expr) => {{
        let result = $client.send($command);
        assert_snapshot!(result, @r###"
        Err(
            HostError(
                ThreadNotSuspended,
            ),
        )
        "###);

        $client.send(Suspend::new($thread))
            .and_then(|_| $client.send($command))
    }};
}

#[test]
fn suspend_resume_status_and_count() -> Result {
    let mut client = common::launch_and_attach("basic")?;

    let main = get_main_thread(&mut client)?;

    let suspend_count = client.send(SuspendCount::new(main))?;
    assert_eq!(suspend_count, 0);

    client.send(Suspend::new(main))?;

    let status = client.send(Status::new(main))?;
    assert_snapshot!(status, @r###"
    (
        Sleeping,
        Suspended,
    )
    "###);

    client.send(Suspend::new(main))?;
    client.send(Suspend::new(main))?;
    client.send(Suspend::new(main))?;

    let suspend_count = client.send(SuspendCount::new(main))?;
    assert_eq!(suspend_count, 4);

    client.send(Resume::new(main))?;
    client.send(Resume::new(main))?;
    client.send(Resume::new(main))?;
    client.send(Resume::new(main))?;

    let status = client.send(Status::new(main))?;
    assert_snapshot!(status, @r###"
    (
        Sleeping,
        NotSuspended,
    )
    "###);

    Ok(())
}

#[test]
fn thread_group() -> Result {
    let mut client = common::launch_and_attach("basic")?;
    let main = get_main_thread(&mut client)?;

    let thread_group = client.send(ThreadGroup::new(main))?;
    let name = client.send(thread_group_reference::Name::new(thread_group))?;

    assert_eq!(name, "main");

    Ok(())
}

#[test]
fn frames() -> Result {
    let mut client = common::launch_and_attach("basic")?;
    let main = get_main_thread(&mut client)?;

    let frames = check_host_suspended!(client, main, Frames::new(main, 0, FrameLimit::Limit(3)))?;

    let mut frame_info = vec![];

    for (frame_id, location) in frames {
        if let TaggedReferenceTypeID::Class(class_id) = location.reference_id {
            let signature = client.send(Signature::new(*class_id))?;

            // meh
            let method = client
                .send(Methods::new(*class_id))?
                .into_iter()
                .find(|m| m.method_id == location.method_id)
                .expect("Didn't find the location method");

            frame_info.push((
                frame_id,
                signature,
                method.name,
                method.signature,
                location.index,
            ));
        } else {
            panic!(
                "Unexpected type of reference id: {:?}",
                location.reference_id
            )
        }
    }

    // thread.sleep is native, not sure if it's location index is stable, CI will
    // tell
    assert_snapshot!(frame_info, @r###"
    [
        (
            FrameID(0),
            "Ljava/lang/Thread;",
            "sleep",
            "(J)V",
            18446744073709551615,
        ),
        (
            FrameID(1),
            "LBasic;",
            "getAsInt",
            "()I",
            52,
        ),
        (
            FrameID(2),
            "LBasic;",
            "main",
            "([Ljava/lang/String;)V",
            3,
        ),
    ]
    "###);

    Ok(())
}

#[test]
fn frame_count() -> Result {
    let mut client = common::launch_and_attach("basic").unwrap();
    let main = get_main_thread(&mut client).unwrap();

    let frame_count = check_host_suspended!(client, main, FrameCount::new(main))?;

    assert_eq!(frame_count, 3);

    Ok(())
}

// todo: make a separate fixture with monitors
#[test]
fn owned_monitors() -> Result {
    let mut client = common::launch_and_attach("basic")?;
    let main = get_main_thread(&mut client)?;

    let owned_monitors = check_host_suspended!(client, main, OwnedMonitors::new(main))?;

    assert_snapshot!(owned_monitors, @"[]");

    Ok(())
}

#[test]
fn current_contended_monitor() -> Result {
    let mut client = common::launch_and_attach("basic")?;
    let main = get_main_thread(&mut client)?;

    let current_contended_monitor =
        check_host_suspended!(client, main, CurrentContendedMonitor::new(main))?;

    assert_snapshot!(current_contended_monitor, @r###"
    Some(
        Object(
            [opaque_id],
        ),
    )
    "###);

    Ok(())
}

#[test]
fn owned_monitors_stack_depth_info() -> Result {
    let mut client = common::launch_and_attach("basic")?;
    let main = get_main_thread(&mut client)?;

    let owned_monitors_stack_depth_info =
        check_host_suspended!(client, main, OwnedMonitorsStackDepthInfo::new(main))?;

    assert_snapshot!(owned_monitors_stack_depth_info, @"[]");

    Ok(())
}

#[test]
fn force_early_return() -> Result {
    let mut client = common::launch_and_attach("basic")?;
    let main = get_main_thread(&mut client)?;

    // we stop at thread.sleep which is a native method
    // todo: make a better test where we stop with an event in a place where we can
    // actually force the return
    let err = check_host_suspended!(client, main, ForceEarlyReturn::new(main, Value::Int(42)));

    assert_snapshot!(err, @r###"
    Err(
        HostError(
            OpaqueFrame,
        ),
    )
    "###);

    Ok(())
}
