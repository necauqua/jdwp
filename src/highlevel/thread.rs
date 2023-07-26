use super::{ExtendedJvmObject, JvmObject, ThreadGroup};
use crate::{
    client::ClientError,
    spec::{
        thread_reference::{self, FrameCount, Name, Resume, Status, Suspend, SuspendCount},
        SuspendStatus, ThreadID, ThreadStatus,
    },
};

pub type JvmThread = ExtendedJvmObject<ThreadID>;

impl JvmThread {
    pub fn name(&self) -> Result<String, ClientError> {
        self.client().get().send(Name::new(self.id()))
    }

    pub fn suspend(&self) -> Result<(), ClientError> {
        self.client().get().send(Suspend::new(self.id()))
    }

    pub fn resume(&self) -> Result<(), ClientError> {
        self.client().get().send(Resume::new(self.id()))
    }

    pub fn status(&self) -> Result<(ThreadStatus, SuspendStatus), ClientError> {
        self.client().get().send(Status::new(self.id()))
    }

    pub fn group(&self) -> Result<ThreadGroup, ClientError> {
        let id = self
            .client()
            .get()
            .send(thread_reference::ThreadGroup::new(self.id()))?;
        Ok(ThreadGroup::new(self.client().clone(), id))
    }

    pub fn frame_count(&self) -> Result<u32, ClientError> {
        self.client().get().send(FrameCount::new(self.id()))
    }

    pub fn suspend_count(&self) -> Result<u32, ClientError> {
        self.client().get().send(SuspendCount::new(self.id()))
    }
}
