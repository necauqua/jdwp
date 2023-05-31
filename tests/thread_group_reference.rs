use std::collections::HashSet;

use common::Result;
use jdwp::commands::{
    thread_group_reference::{Children, Name, Parent},
    thread_reference,
    virtual_machine::TopLevelThreadGroups,
};

mod common;

#[test]
fn system_tree_names() -> Result {
    let mut client = common::launch_and_attach("basic")?;

    let thread_group_ids = client.send(TopLevelThreadGroups).unwrap();
    assert_eq!(thread_group_ids.len(), 1);
    let thread_group = thread_group_ids[0];

    let children = client.send(Children::new(thread_group))?;

    let parent_names = children
        .child_groups
        .iter()
        .map(|id| {
            let parent = client
                .send(Parent::new(*id))?
                .expect("Thread Group Parent was None");
            let name = client.send(Name::new(parent))?;
            Ok(name)
        })
        .collect::<Result<HashSet<_>>>()?;

    let child_names = children
        .child_groups
        .iter()
        .map(|id| Ok(client.send(Name::new(*id))?))
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .filter(|name| name != "InnocuousThreadGroup") // not present on jdk8
        .collect::<Vec<_>>();

    let thread_names = children
        .child_threads
        .iter()
        .map(|id| Ok(client.send(thread_reference::Name::new(*id))?))
        .collect::<Result<Vec<_>>>()?;

    let expected_threads = &["Signal Dispatcher", "Reference Handler", "Finalizer"];

    assert!(thread_names.len() >= expected_threads.len());
    assert!(thread_names
        .iter()
        .any(|n| expected_threads.contains(&n.as_str())));

    assert_snapshot!((parent_names, child_names), @r###"
    (
        {
            "system",
        },
        [
            "main",
        ],
    )
    "###);

    Ok(())
}
