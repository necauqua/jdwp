use std::collections::HashSet;

use common::Result;

mod common;

#[test]
fn system_tree_names() -> Result {
    let vm = common::launch_and_attach_vm("basic")?;

    let thread_groups = vm.top_level_thread_groups()?;
    assert_eq!(thread_groups.len(), 1);
    let thread_group = thread_groups.into_iter().next().unwrap();

    let (child_groups, threads) = thread_group.children()?;

    let parent_names = child_groups
        .iter()
        .map(|group| {
            let parent = group.parent()?.expect("Thread Group Parent was None");
            Ok(parent.name()?)
        })
        .collect::<Result<HashSet<_>>>()?;

    let child_names = child_groups
        .iter()
        .map(|group| Ok(group.name()?))
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .filter(|name| name != "InnocuousThreadGroup") // not present on jdk8
        .collect::<Vec<_>>();

    let thread_names = threads
        .iter()
        .map(|thread| Ok(thread.name()?))
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
