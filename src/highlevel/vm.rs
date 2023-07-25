use std::{fmt::Debug, net::ToSocketAddrs};

use super::{
    JvmEvent, JvmString, JvmThread, PlainJvmObject, SharedClient, TaggedReferenceType, ThreadGroup,
};
use crate::{
    client::{ClientError, JdwpClient},
    codec::{JdwpReadable, JdwpWritable},
    functional::Coll,
    spec::{
        virtual_machine::{
            self, AllClasses, AllClassesWithGeneric, CapabilitiesNewReply, CapabilitiesReply,
            ClassBySignature, ClassPathsReply, ClassesBySignature, IDSizeInfo, InstanceCounts,
            Version, VersionReply,
        },
        ClassStatus, Command, JdwpId, ReferenceTypeID, SuspendPolicy,
    },
};

#[derive(Debug)]
pub struct VM {
    client: SharedClient,
}

impl From<JdwpClient> for VM {
    fn from(client: JdwpClient) -> Self {
        Self::new(client)
    }
}

impl VM {
    pub fn new(client: impl Into<SharedClient>) -> Self {
        Self {
            client: client.into(),
        }
    }

    pub fn connect(addr: impl ToSocketAddrs) -> Result<Self, ClientError> {
        Ok(Self::new(JdwpClient::connect(addr)?))
    }

    pub fn child<I: JdwpId>(&self, id: I) -> PlainJvmObject<I> {
        PlainJvmObject::new(self.client.clone(), id)
    }

    pub fn client(&self) -> &SharedClient {
        &self.client
    }
}

impl VM {
    pub fn version(&self) -> Result<VersionReply, ClientError> {
        self.client.get().send(Version)
    }

    pub fn class_by_signature(
        &self,
        signature: &str,
    ) -> Result<(TaggedReferenceType, ClassStatus), ClientError> {
        let (id, status) = *self.client.get().send(ClassBySignature::new(signature))?;
        Ok((TaggedReferenceType::new(self.client.clone(), id), status))
    }

    pub fn classes_by_signature(
        &self,
        signature: &str,
    ) -> Result<Vec<(TaggedReferenceType, ClassStatus)>, ClientError> {
        let classes = self.client.get().send(ClassesBySignature::new(signature))?;
        let classes = classes
            .into_iter()
            .map(|(id, status)| (TaggedReferenceType::new(self.client.clone(), id), status))
            .collect();
        Ok(classes)
    }

    pub fn all_classes(&self) -> Result<Vec<Class>, ClientError> {
        let classes = self.client.get().send(AllClasses)?;
        let classes = classes
            .into_iter()
            .map(|class| Class {
                object: TaggedReferenceType::new(self.client.clone(), class.type_id),
                signature: class.signature,
                generic_signature: None,
                status: class.status,
            })
            .collect();
        Ok(classes)
    }

    pub fn all_threads(&self) -> Result<Vec<JvmThread>, ClientError> {
        let reply = self.client.get().send(virtual_machine::AllThreads)?;
        Ok(reply
            .iter()
            .map(|id| JvmThread::new(self.client.clone(), *id))
            .collect())
    }

    pub fn top_level_thread_groups(&self) -> Result<Vec<ThreadGroup>, ClientError> {
        let reply = self
            .client
            .get()
            .send(virtual_machine::TopLevelThreadGroups)?;
        Ok(reply
            .iter()
            .map(|id| ThreadGroup::new(self.client.clone(), *id))
            .collect())
    }

    pub fn dispose(self) -> Result<(), ClientError> {
        self.client.get().send(virtual_machine::Dispose)
    }

    pub fn id_sizes(&self) -> Result<IDSizeInfo, ClientError> {
        self.client.get().send(virtual_machine::IDSizes)
    }

    pub fn suspend(&self) -> Result<(), ClientError> {
        self.client.get().send(virtual_machine::Suspend)
    }

    pub fn resume(&self) -> Result<(), ClientError> {
        self.client.get().send(virtual_machine::Resume)
    }

    pub fn exit(self, exit_code: i32) -> Result<(), ClientError> {
        self.client
            .get()
            .send(virtual_machine::Exit::new(exit_code))
    }

    pub fn create_string(&self, value: &str) -> Result<JvmString, ClientError> {
        let id = self
            .client
            .get()
            .send(virtual_machine::CreateString::new(value))?;
        Ok(JvmString::new(self.client.clone(), id))
    }

    pub fn capabilities(&self) -> Result<CapabilitiesReply, ClientError> {
        self.client.get().send(virtual_machine::Capabilities)
    }

    pub fn classpaths(&self) -> Result<ClassPathsReply, ClientError> {
        self.client.get().send(virtual_machine::ClassPaths)
    }

    pub fn hold_events(&self) -> Result<(), ClientError> {
        self.client.get().send(virtual_machine::HoldEvents)
    }

    pub fn release_events(&self) -> Result<(), ClientError> {
        self.client.get().send(virtual_machine::ReleaseEvents)
    }

    pub fn capabilities_new(&self) -> Result<CapabilitiesNewReply, ClientError> {
        self.client.get().send(virtual_machine::CapabilitiesNew)
    }

    pub fn set_default_stratum(&self, stratum: &str) -> Result<(), ClientError> {
        self.client
            .get()
            .send(virtual_machine::SetDefaultStratum::new(stratum))
    }

    pub fn all_classes_with_generic(&self) -> Result<Vec<Class>, ClientError> {
        let classes = self.client.get().send(AllClassesWithGeneric)?;
        let classes = classes
            .into_iter()
            .map(|class| Class {
                object: TaggedReferenceType::new(self.client.clone(), class.type_id),
                signature: class.signature,
                generic_signature: class.generic_signature,
                status: class.status,
            })
            .collect();
        Ok(classes)
    }

    // oof wtf are those bounds
    pub fn instance_counts<C: Coll<Item = ReferenceTypeID>>(
        &self,
        ref_types: C,
    ) -> Result<C::Map<u64>, ClientError>
    where
        InstanceCounts<C>: JdwpWritable + Debug,
        <InstanceCounts<C> as Command>::Output: JdwpReadable + Debug,
    {
        self.client.get().send(InstanceCounts::new(ref_types))
    }
}

impl VM {
    pub fn main_thread(&self) -> Result<JvmThread, ClientError> {
        for thread in self.all_threads()? {
            if thread.name()? == "main" {
                return Ok(thread);
            }
        }
        todo!("high level errors")
    }

    pub fn receive_events(&self) -> impl Iterator<Item = (SuspendPolicy, Vec<JvmEvent>)> {
        let cloned = self.client.clone(); // avoid return depending on the lifetime of &self
        self.client
            .get()
            .receive_events()
            .into_iter()
            .map(move |composite| {
                let events = composite
                    .events
                    .into_iter()
                    .map(|event| JvmEvent::new(event, cloned.clone()))
                    .collect();
                (composite.suspend_policy, events)
            })
    }
}

#[derive(Debug)]
pub struct Class {
    pub object: TaggedReferenceType,
    pub signature: String,
    pub generic_signature: Option<String>,
    pub status: ClassStatus,
}
