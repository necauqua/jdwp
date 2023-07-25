use crate::{
    client::ClientError,
    spec::{class_type::Superclass, ClassID},
};

use super::{ExtendedJvmObject, JvmObject};

pub type ClassType = ExtendedJvmObject<ClassID>;

impl ClassType {
    pub fn superclass(&self) -> Result<Option<ClassType>, ClientError> {
        let mut client = self.client().get();
        Ok(client
            .send(Superclass::new(self.id()))?
            .map(|id| ClassType::new(self.client().clone(), id)))
    }
}
