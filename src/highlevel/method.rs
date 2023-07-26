use crate::{
    client::ClientError,
    spec::{
        class_type, interface_type, object_reference, ClassID, InvokeMethodReply, InvokeOptions,
        MethodID, Value,
    },
};

use super::{
    ChildJvmObject, ClassType, InterfaceType, JvmObject, JvmThread, ObjectReference, SharedClient,
    TaggedObject,
};

pub type JvmClassMethod = ChildJvmObject<ClassType, MethodID>;
pub type JvmInterfaceMethod = ChildJvmObject<InterfaceType, MethodID>;
pub type JvmInstanceMethod = ChildJvmObject<ObjectReference, (ClassID, MethodID)>;

macro_rules! invoke_impls {
    ($($module:ident => $method_type:ty)*) => {
        $(
            impl $method_type {
                pub fn invoke(
                    &self,
                    thread: JvmThread,
                    args: &[Value],
                    options: InvokeOptions,
                ) -> Result<InvokeReply, ClientError> {
                    let reply = self.client().get().send($module::InvokeMethod::new(
                        self.parent().id(),
                        thread.id(),
                        self.id(),
                        args,
                        options,
                    ))?;
                    Ok(InvokeReply::new(self.client().clone(), reply))
                }
            }
        )*
    };
}

invoke_impls! {
    class_type => JvmClassMethod
    interface_type => JvmInterfaceMethod
    object_reference => JvmInstanceMethod
}

#[derive(Debug)]
pub enum InvokeReply {
    Value(Value),
    Exception(TaggedObject),
}

impl InvokeReply {
    pub fn new(client: impl Into<SharedClient>, reply: InvokeMethodReply) -> Self {
        match reply {
            InvokeMethodReply::Value(v) => Self::Value(v),
            InvokeMethodReply::Exception(obj) => {
                Self::Exception(TaggedObject::new(client.into(), obj))
            }
        }
    }
}
