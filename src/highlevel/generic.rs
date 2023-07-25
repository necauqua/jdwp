use std::{
    fmt::{self, Debug},
    ops::Deref,
};

use super::SharedClient;

pub trait JvmObject: Clone {
    type Id;

    fn client(&self) -> &SharedClient;

    fn id(&self) -> Self::Id;

    fn child<N: Clone>(&self, id: N) -> ChildJvmObject<Self, N> {
        ChildJvmObject::new(self.clone(), id)
    }
}

#[derive(Clone)]
pub struct PlainJvmObject<I> {
    client: SharedClient,
    id: I,
}

impl<I: Clone> JvmObject for PlainJvmObject<I> {
    type Id = I;

    fn id(&self) -> I {
        self.id.clone()
    }

    fn client(&self) -> &SharedClient {
        &self.client
    }
}

impl<I: Clone> PlainJvmObject<I> {
    pub fn new(client: impl Into<SharedClient>, id: I) -> Self {
        Self {
            client: client.into(),
            id,
        }
    }
}

impl<I: Debug> Debug for PlainJvmObject<I> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("JvmObject").field(&self.id).finish()
    }
}

#[derive(Clone)]
pub struct ChildJvmObject<P: JvmObject, I> {
    parent: P,
    id: I,
}

impl<P: JvmObject, I: Clone> JvmObject for ChildJvmObject<P, I> {
    type Id = I;

    fn id(&self) -> I {
        self.id.clone()
    }

    fn client(&self) -> &SharedClient {
        self.parent.client()
    }
}

impl<P: JvmObject, I: Clone> ChildJvmObject<P, I> {
    pub fn new(parent: P, id: I) -> Self {
        Self { parent, id }
    }

    pub fn parent(&self) -> &P {
        &self.parent
    }
}

impl<P: JvmObject, I: Debug> Debug for ChildJvmObject<P, I>
where
    P::Id: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("NestedJvmObject")
            .field(&self.parent.id())
            .field(&self.id)
            .finish()
    }
}

#[derive(Clone)]
pub struct ExtendedJvmObject<I>
where
    I: Deref,
    I::Target: Sized,
{
    peer: PlainJvmObject<I::Target>,
    id: I,
}

impl<I> JvmObject for ExtendedJvmObject<I>
where
    I: Clone + Deref,
    I::Target: Clone,
{
    type Id = I;

    fn id(&self) -> I {
        self.id.clone()
    }

    fn client(&self) -> &SharedClient {
        &self.peer.client
    }
}

impl<I> ExtendedJvmObject<I>
where
    I: Clone + Deref,
    I::Target: Clone,
{
    pub fn new(client: impl Into<SharedClient>, id: I) -> Self {
        Self {
            peer: PlainJvmObject::new(client, (*id).clone()),
            id,
        }
    }

    pub fn peer(&self) -> &PlainJvmObject<I::Target> {
        &self.peer
    }
}

impl<I: Debug> Debug for ExtendedJvmObject<I>
where
    I: Deref,
    I::Target: Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("WrapperJvmObject").field(&self.id).finish()
    }
}

impl<I> From<PlainJvmObject<I>> for ExtendedJvmObject<I>
where
    I: Clone + Deref,
    I::Target: Clone,
{
    fn from(peer: PlainJvmObject<I>) -> Self {
        Self::new(peer.client, peer.id)
    }
}

impl<I> Deref for ExtendedJvmObject<I>
where
    I: Deref,
    I::Target: Sized,
{
    type Target = PlainJvmObject<I::Target>;

    fn deref(&self) -> &Self::Target {
        &self.peer
    }
}
