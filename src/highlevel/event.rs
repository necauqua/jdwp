use crate::{
    highlevel::JvmField,
    spec::event::{self, Spec},
};

use super::{JvmThread, SharedClient, TaggedObject, TaggedReferenceType};

#[derive(Debug, Clone)]
pub struct Object;

impl event::sealed::Domain for Object {
    type Thread = JvmThread;
    type TaggedObject = TaggedObject;
    type TaggedReferenceType = TaggedReferenceType;
    type Field = JvmField;
}

pub type JvmEvent = event::Event<Object>;

impl JvmEvent {
    pub fn new(event: event::Event<Spec>, client: SharedClient) -> Self {
        use event::Event::*;

        match event {
            SingleStep(rid, tid, loc) => SingleStep(rid, JvmThread::new(client, tid), loc),
            Breakpoint(rid, tid, loc) => Breakpoint(rid, JvmThread::new(client, tid), loc),
            MethodEntry(rid, tid, loc) => MethodEntry(rid, JvmThread::new(client, tid), loc),
            MethodExit(rid, tid, loc) => MethodExit(rid, JvmThread::new(client, tid), loc),
            MethodExitWithReturnValue(rid, tid, loc, val) => {
                MethodExitWithReturnValue(rid, JvmThread::new(client, tid), loc, val)
            }
            MonitorContendedEnter(rid, tid, obj, loc) => {
                let thread = JvmThread::new(client.clone(), tid);
                let object = TaggedObject::new(client, obj);
                MonitorContendedEnter(rid, thread, object, loc)
            }
            MonitorContendedEntered(rid, tid, obj, loc) => {
                let thread = JvmThread::new(client.clone(), tid);
                let object = TaggedObject::new(client, obj);
                MonitorContendedEntered(rid, thread, object, loc)
            }
            MonitorWait(rid, tid, obj, loc, time) => {
                let thread = JvmThread::new(client.clone(), tid);
                let object = TaggedObject::new(client, obj);
                MonitorWait(rid, thread, object, loc, time)
            }
            MonitorWaited(rid, tid, obj, loc, timed_out) => {
                let thread = JvmThread::new(client.clone(), tid);
                let object = TaggedObject::new(client, obj);
                MonitorWaited(rid, thread, object, loc, timed_out)
            }
            Exception(rid, tid, e_loc, obj, c_loc) => {
                let thread = JvmThread::new(client.clone(), tid);
                let object = TaggedObject::new(client, obj);
                Exception(rid, thread, e_loc, object, c_loc)
            }
            ThreadStart(rid, tid) => ThreadStart(rid, JvmThread::new(client, tid)),
            ThreadDeath(rid, tid) => ThreadDeath(rid, JvmThread::new(client, tid)),
            ClassPrepare(rid, tid, ref_id, sig, st) => {
                let thread = JvmThread::new(client.clone(), tid);
                let ref_type = TaggedReferenceType::new(client, ref_id);
                ClassPrepare(rid, thread, ref_type, sig, st)
            }
            ClassUnload(rid, sig) => ClassUnload(rid, sig),
            FieldAccess(rid, tid, loc, field_data) => {
                let thread = JvmThread::new(client.clone(), tid);
                let field = JvmField::new(client, field_data);
                FieldAccess(rid, thread, loc, field)
            }
            FieldModification(rid, tid, loc, field_data, val) => {
                let thread = JvmThread::new(client.clone(), tid);
                let field = JvmField::new(client, field_data);
                FieldModification(rid, thread, loc, field, val)
            }
            VmStart(rid, tid) => VmStart(rid, JvmThread::new(client, tid)),
            VmDeath(rid) => VmDeath(rid),
        }
    }
}
