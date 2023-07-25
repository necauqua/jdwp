use std::fmt::Debug;

use crate::{
    client::ClientError,
    codec::JdwpReadable,
    functional::{Coll, Single},
    spec::{
        class_type, object_reference, reference_type, FieldID, ReferenceTypeID, TaggedObjectID,
        TaggedReferenceTypeID, UntaggedValue, Value,
    },
};

use super::{
    ChildJvmObject, ClassType, JvmObject, ObjectReference, PlainJvmObject, ReferenceType,
    SharedClient, TaggedReferenceType,
};

pub type JvmReferenceField = ChildJvmObject<ReferenceType, FieldID>;
pub type JvmReferenceFields<const N: usize> = ChildJvmObject<ReferenceType, [FieldID; N]>;

impl JvmReferenceField {
    pub fn get(&self) -> Result<Value, ClientError> {
        let res = self.client().get().send(reference_type::GetValues::new(
            self.parent().id(),
            Single(self.id()),
        ))?;
        Ok(*res)
    }
}

impl<C> ChildJvmObject<PlainJvmObject<ReferenceTypeID>, C>
where
    C: Coll<Item = FieldID> + Clone + Debug,
    C::Map<Value>: JdwpReadable + Debug,
{
    pub fn get(&self) -> Result<C::Map<Value>, ClientError> {
        let res = self.client().get().send(reference_type::GetValues::new(
            self.parent().id(),
            self.id(),
        ))?;
        Ok(res)
    }
}

pub type JvmClassField = ChildJvmObject<ClassType, FieldID>;
pub type JvmClassFields<const N: usize> = ChildJvmObject<ClassType, [FieldID; N]>;

impl JvmClassField {
    pub fn get(&self) -> Result<Value, ClientError> {
        let res = self.client().get().send(reference_type::GetValues::new(
            *self.parent().id(),
            Single(self.id()),
        ))?;
        Ok(*res)
    }

    pub fn set(&self, value: impl Into<UntaggedValue>) -> Result<(), ClientError> {
        self.client().get().send(class_type::SetValues::new(
            self.parent().id(),
            &[(self.id(), value.into())],
        ))
    }
}

impl<const N: usize> JvmClassFields<N> {
    pub fn get(&self) -> Result<[Value; N], ClientError> {
        self.client().get().send(reference_type::GetValues::new(
            *self.parent().id(),
            self.id(),
        ))
    }

    pub fn set(&self, values: [impl Into<UntaggedValue>; N]) -> Result<(), ClientError> {
        self.client().get().send(class_type::SetValues::new(
            self.parent().id(),
            &self
                .id()
                .into_iter()
                .zip(values.into_iter().map(Into::into))
                .collect::<Vec<_>>()[..],
        ))
    }
}

pub type JvmInstanceField = ChildJvmObject<ObjectReference, FieldID>;
pub type JvmInstanceFields<const N: usize> = ChildJvmObject<ObjectReference, [FieldID; N]>;

impl JvmInstanceField {
    pub fn get(&self) -> Result<Value, ClientError> {
        let [value] = self.client().get().send(object_reference::GetValues::new(
            self.parent().id(),
            [self.id()],
        ))?;
        Ok(value)
    }

    pub fn set(&self, value: impl Into<UntaggedValue>) -> Result<(), ClientError> {
        self.client().get().send(object_reference::SetValues::new(
            self.parent().id(),
            &[(self.id(), value.into())],
        ))
    }
}

impl<const N: usize> JvmInstanceFields<N> {
    pub fn get(&self) -> Result<[Value; N], ClientError> {
        self.client().get().send(object_reference::GetValues::new(
            self.parent().id(),
            self.id(),
        ))
    }

    pub fn set(&self, values: [impl Into<UntaggedValue>; N]) -> Result<(), ClientError> {
        self.client().get().send(object_reference::SetValues::new(
            self.parent().id(),
            &self
                .id()
                .into_iter()
                .zip(values.into_iter().map(Into::into))
                .collect::<Vec<_>>()[..],
        ))
    }
}

#[derive(Debug)]
pub enum JvmField {
    /// If the static field is part of an array or interface type, there is no
    /// way to set it.
    ReadonlyStatic(JvmReferenceField),
    /// A static field of a class.
    Static(JvmClassField),
    /// An instance field of an object.
    Instance(JvmInstanceField),
}

impl JvmField {
    pub fn new(
        client: impl Into<SharedClient>,
        (ref_id, fid, obj): (TaggedReferenceTypeID, FieldID, Option<TaggedObjectID>),
    ) -> Self {
        if let Some(obj) = obj {
            Self::Instance(ObjectReference::new(client, *obj).child(fid))
        } else if let TaggedReferenceTypeID::Class(class_id) = ref_id {
            Self::Static(ClassType::new(client, class_id).child(fid))
        } else {
            Self::ReadonlyStatic(TaggedReferenceType::new(client, ref_id).child(fid))
        }
    }

    /// A shortcut to get the value of any kind of field.
    pub fn get(&self) -> Result<Value, ClientError> {
        match self {
            Self::ReadonlyStatic(field) => field.get(),
            Self::Static(field) => field.get(),
            Self::Instance(field) => field.get(),
        }
    }

    /// A shortcut to set the value of any kind of field.
    ///
    /// If the field is a static reference field, that is it's not statically
    /// verified to be a static class field, this will fail.
    pub fn set(&self, value: impl Into<UntaggedValue>) -> Result<(), ClientError> {
        match self {
            Self::ReadonlyStatic(_) => todo!("highlevel errors"),
            Self::Static(field) => field.set(value),
            Self::Instance(field) => field.set(value),
        }
    }
}
