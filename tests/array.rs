use common::Result;

mod common;

#[test]
fn length() -> Result {
    let vm = common::launch_and_attach_vm("basic")?;
    let (class_type, _) = vm.class_by_signature("[I")?;

    let array_type = class_type.unwrap_array();
    let array = array_type.new_instance(10)?;

    assert_eq!(array.length()?, 10);

    Ok(())
}

#[test]
fn set_get_values() -> Result {
    let vm = common::launch_and_attach_vm("basic")?;
    let (class_type, _) = vm.class_by_signature("[I")?;

    let array_type = class_type.unwrap_array();
    let array = array_type.new_instance(10)?;

    array.set_values(0, &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10])?;

    let region = array.get_values(0, 10)?;

    assert_snapshot!(region, @r###"
    Int(
        [
            1,
            2,
            3,
            4,
            5,
            6,
            7,
            8,
            9,
            10,
        ],
    )
    "###);

    Ok(())
}
