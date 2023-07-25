use crate::{
    client::ClientError,
    spec::{array_type::NewInstance, ArrayTypeID},
};

use super::{ExtendedJvmObject, JvmArray, JvmObject};

pub type ArrayType = ExtendedJvmObject<ArrayTypeID>;

impl ArrayType {
    pub fn new_instance(&self, length: u32) -> Result<JvmArray, ClientError> {
        let array = self
            .client()
            .get()
            .send(NewInstance::new(self.id(), length))?;
        Ok(JvmArray::new(self.client().clone(), array.new_array))
    }
}
