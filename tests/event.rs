use jdwp::{
    commands::{
        event::Event, event_request, reference_type::Fields, virtual_machine::ClassesBySignature,
    },
    enums::{EventKind, SuspendPolicy},
    types::{FieldOnly, Modifier, Value},
};

mod common;

use common::Result;

#[test]
fn field_modification() -> Result {
    let mut client = common::launch_and_attach("basic")?;

    let type_id = client.send(ClassesBySignature::new("LBasic;"))?[0].type_id;

    let ticks = &client
        .send(Fields::new(*type_id))?
        .into_iter()
        .find(|f| f.name == "ticks")
        .unwrap();

    let request_id = client.send(event_request::Set::new(
        EventKind::FieldModification,
        SuspendPolicy::None,
        vec![Modifier::FieldOnly(FieldOnly {
            declaring: *type_id,
            field_id: ticks.field_id,
        })],
    ))?;

    match &client.host_events().recv()?.events[..] {
        [Event::FieldModification(field_modification)] => {
            assert_eq!(field_modification.request_id, request_id);

            // not sure if the main thread is always 1
            // assert_eq!(field_modification.thread, unsafe { ThreadID::new(1) });

            // should be modified from own class
            assert_eq!(field_modification.ref_type_id, type_id);
            assert_eq!(field_modification.field_id, ticks.field_id);
            // field is non-static
            assert!(field_modification.object.is_some());

            // check if it was a long and if it did tick
            assert!(matches!(field_modification.value, Value::Long(x) if x >= 1));
        }
        e => panic!("Unexpected event set received: {:#?}", e),
    }

    client.send(event_request::Clear::new(
        EventKind::FieldModification,
        request_id,
    ))?;

    Ok(())
}
