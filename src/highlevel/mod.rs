use std::{
    ops::Deref,
    sync::{Arc, Mutex, MutexGuard},
};

use crate::client::JdwpClient;

#[derive(Debug, Clone)]
pub struct SharedClient(Arc<Mutex<JdwpClient>>);

impl From<JdwpClient> for SharedClient {
    fn from(client: JdwpClient) -> Self {
        Self(Arc::new(Mutex::new(client)))
    }
}

impl SharedClient {
    fn get(&self) -> MutexGuard<JdwpClient> {
        self.0.lock().expect("Posioned client lock")
    }
}

impl Deref for SharedClient {
    type Target = Arc<Mutex<JdwpClient>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

mod generic;
pub use generic::*;

mod vm;
pub use vm::*;

mod reference_type;
pub use reference_type::*;

mod thread;
pub use thread::*;

mod class_type;
pub use class_type::*;

mod interface_type;
pub use interface_type::*;

mod array_type;
pub use array_type::*;

mod object_reference;
pub use object_reference::*;

mod field;
pub use field::*;

mod method;
pub use method::*;

mod event;
pub use event::*;
