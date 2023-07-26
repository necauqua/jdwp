use crate::{
    client::ClientError,
    spec::{
        array_reference::{GetValues, Length, SetValues},
        class_loader_reference::VisibleClasses,
        class_object_reference::ReflectedType,
        string_reference::Value,
        thread_group_reference::{self, Children, Parent},
        virtual_machine::DisposeObjects,
        ArrayID, ArrayRegion, ClassLoaderID, ClassObjectID, JdwpValue, ObjectID, StringID, Tag,
        TaggedObjectID, ThreadGroupID,
    },
};
use std::{fmt::Debug, ops::Deref};

use super::{
    ExtendedJvmObject, JvmObject, JvmThread, PlainJvmObject, SharedClient, TaggedReferenceType,
};

pub type ObjectReference = PlainJvmObject<ObjectID>;

impl ObjectReference {
    pub fn dispose_single(&self) -> Result<(), ClientError> {
        self.dispose(1)
    }

    pub fn dispose(&self, refcount: u32) -> Result<(), ClientError> {
        self.client()
            .get()
            .send(DisposeObjects::new(&[(self.id(), refcount)]))
    }
}

pub type JvmArray = ExtendedJvmObject<ArrayID>;

impl JvmArray {
    pub fn length(&self) -> Result<u32, ClientError> {
        self.client().get().send(Length::new(self.id()))
    }

    pub fn get_values(&self, first_index: u32, length: u32) -> Result<ArrayRegion, ClientError> {
        self.client()
            .get()
            .send(GetValues::new(self.id(), first_index, length))
    }

    pub fn set_values(
        &self,
        first_index: u32,
        values: &[impl JdwpValue + Debug],
    ) -> Result<(), ClientError> {
        self.client()
            .get()
            .send(SetValues::new(self.id(), first_index, values))
    }
}

pub type JvmString = ExtendedJvmObject<StringID>;

impl JvmString {
    pub fn value(&self) -> Result<String, ClientError> {
        self.client().get().send(Value::new(self.id()))
    }
}

pub type ThreadGroup = ExtendedJvmObject<ThreadGroupID>;

impl ThreadGroup {
    pub fn name(&self) -> Result<String, ClientError> {
        self.client()
            .get()
            .send(thread_group_reference::Name::new(self.id()))
    }

    pub fn parent(&self) -> Result<Option<ThreadGroup>, ClientError> {
        let parent_id = self.client().get().send(Parent::new(self.id()))?;
        Ok(parent_id.map(|id| ThreadGroup::new(self.client().clone(), id)))
    }

    pub fn children(&self) -> Result<(Vec<ThreadGroup>, Vec<JvmThread>), ClientError> {
        let reply = self.client().get().send(Children::new(self.id()))?;
        let groups = reply
            .child_groups
            .iter()
            .map(|id| ThreadGroup::new(self.client().clone(), *id))
            .collect();
        let threads = reply
            .child_threads
            .iter()
            .map(|id| JvmThread::new(self.client().clone(), *id))
            .collect();
        Ok((groups, threads))
    }
}

pub type ClassLoader = ExtendedJvmObject<ClassLoaderID>;

impl ClassLoader {
    pub fn visible_classes(&self) -> Result<Vec<TaggedReferenceType>, ClientError> {
        let classes = self.client().get().send(VisibleClasses::new(self.id()))?;
        Ok(classes
            .into_iter()
            .map(|id| TaggedReferenceType::new(self.client().clone(), id))
            .collect())
    }
}

pub type ClassObject = ExtendedJvmObject<ClassObjectID>;

impl ClassObject {
    pub fn reflected_type(&self) -> Result<TaggedReferenceType, ClientError> {
        let ref_type = self.client().get().send(ReflectedType::new(self.id()))?;
        Ok(TaggedReferenceType::new(self.client().clone(), ref_type))
    }
}

#[derive(Debug)]
#[repr(u8)]
pub enum TaggedObject {
    Array(JvmArray) = Tag::Array as u8,
    Object(ObjectReference) = Tag::Object as u8,
    String(JvmString) = Tag::String as u8,
    Thread(JvmThread) = Tag::Thread as u8,
    ThreadGroup(ThreadGroup) = Tag::ThreadGroup as u8,
    ClassLoader(ClassLoader) = Tag::ClassLoader as u8,
    ClassObject(ClassObject) = Tag::ClassObject as u8,
}

impl TaggedObject {
    pub fn new(client: impl Into<SharedClient>, id: TaggedObjectID) -> Self {
        use TaggedObject::*;
        use TaggedObjectID as ID;

        match id {
            ID::Array(id) => Array(JvmArray::new(client, id)),
            ID::Object(id) => Object(ObjectReference::new(client, id)),
            ID::String(id) => String(JvmString::new(client, id)),
            ID::Thread(id) => Thread(JvmThread::new(client, id)),
            ID::ThreadGroup(id) => ThreadGroup(super::ThreadGroup::new(client, id)),
            ID::ClassLoader(id) => ClassLoader(super::ClassLoader::new(client, id)),
            ID::ClassObject(id) => ClassObject(super::ClassObject::new(client, id)),
        }
    }

    pub fn tag(&self) -> Tag {
        // SAFETY: Self and Tag fulfill the requirements
        unsafe { crate::spec::tag(self) }
    }
}

impl Deref for TaggedObject {
    type Target = ObjectReference;

    fn deref(&self) -> &Self::Target {
        use TaggedObject::*;

        match self {
            Array(obj) => obj,
            Object(obj) => obj,
            String(obj) => obj,
            Thread(obj) => obj,
            ThreadGroup(obj) => obj,
            ClassLoader(obj) => obj,
            ClassObject(obj) => obj,
        }
    }
}
