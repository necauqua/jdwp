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

    let ticks = &client.send(Fields::new(*type_id))?[0];

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

            // 1..20 is because the field is updated every 50ms, prevent flakiness
            // on my machine it's consistently 4
            assert!(matches!(field_modification.value, Value::Long(x) if (1..20).contains(&x)));
        }
        e => panic!("Unexpected event set received: {:#?}", e),
    }

    client.send(event_request::Clear::new(
        EventKind::FieldModification,
        request_id,
    ))?;

    Ok(())
}
