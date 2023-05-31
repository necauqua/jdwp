use jdwp::{
    commands::{
        event::Event,
        event_request,
        reference_type::{Fields, Methods},
        thread_reference,
        virtual_machine::{AllThreads, ClassBySignature},
    },
    enums::{EventKind, SuspendPolicy},
    event_modifier::Modifier,
    types::Value,
};

mod common;

use common::Result;

#[test]
fn field_modification() -> Result {
    let mut client = common::launch_and_attach("basic")?;

    let (type_id, _) = *client.send(ClassBySignature::new("LBasic;"))?;

    let main_thread = client
        .send(AllThreads)?
        .into_iter()
        .map(|id| Ok((id, client.send(thread_reference::Name::new(id))?)))
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .find(|(_, name)| name == "main")
        .unwrap()
        .0;

    let ticks = &client
        .send(Fields::new(*type_id))?
        .into_iter()
        .find(|f| f.name == "ticks")
        .unwrap();

    let tick = &client
        .send(Methods::new(*type_id))?
        .into_iter()
        .find(|m| m.name == "tick")
        .unwrap();

    let field_only = Modifier::FieldOnly(*type_id, ticks.field_id);

    let request_id = client.send(event_request::Set::new(
        EventKind::FieldModification,
        SuspendPolicy::None,
        &[field_only],
    ))?;

    match &client.host_events().recv()?.events[..] {
        [Event::FieldModification(req_id, tid, loc, (rid, fid), oid, v)] => {
            assert_eq!(*req_id, request_id);

            // should be modified in main thread
            assert_eq!(*tid, main_thread);

            // is our field
            assert_eq!((*rid, *fid), (type_id, ticks.field_id));

            // modified from our class
            assert_eq!(loc.reference_id, type_id);
            // from the tick method
            assert_eq!(loc.method_id, tick.method_id);

            // field is non-static
            assert!(oid.is_some());

            // check if it was a long and if it did tick
            assert!(matches!(*v, Value::Long(x) if x >= 1));
        }
        e => panic!("Unexpected event set received: {:#?}", e),
    }

    client.send(event_request::Clear::new(
        EventKind::FieldModification,
        request_id,
    ))?;

    Ok(())
}
