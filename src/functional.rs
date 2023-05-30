use std::{iter, num::NonZeroUsize};

use std::ops::Deref;

mod sealed {
    pub trait Sealed {}
}

/// This trait exists to abstract over Vec<T> and [T; N].
///
/// It makes it convenient to call certain commands where the input list size
/// matches the output list size, for example
/// [GetValues](crate::commands::object_reference::GetValues).
pub trait Coll: sealed::Sealed {
    type Item;
    type Map<U>;
    type Iter<'a>: Iterator<Item = &'a Self::Item>
    where
        Self: 'a;

    const STATIC_SIZE: Option<NonZeroUsize>;

    fn size(&self) -> usize;

    fn iter(&self) -> Self::Iter<'_>;
}

/// This is a "single element collection" wrapper type, in terms of the Coll
/// trait implementation and usage.
///
/// It's only really usable when the command return contains a collection that
/// is not matched by the input argument, when the input argument is matched the
/// easier syntax is to use a 1-sized static array and a descruturing let.
///
/// The best syntax for that would be to have some IntoColl trait that is
/// implemented for T to make Single<T>, but for it to handle [T; N] and Vec<T>
/// we need specialization
#[repr(transparent)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Single<T>(pub T);

impl<T> Deref for Single<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> TryFrom<Vec<T>> for Single<T> {
    type Error = Vec<T>;

    fn try_from(mut vec: Vec<T>) -> Result<Self, Self::Error> {
        if vec.len() == 1 {
            Ok(Single(vec.remove(0)))
        } else {
            Err(vec)
        }
    }
}

impl<T> sealed::Sealed for Single<T> {}
impl<T> Coll for Single<T> {
    type Item = T;
    type Map<U> = Single<U>;
    type Iter<'a> = iter::Once<&'a T> where T: 'a;

    const STATIC_SIZE: Option<NonZeroUsize> = NonZeroUsize::new(1);

    fn size(&self) -> usize {
        1
    }

    fn iter(&self) -> Self::Iter<'_> {
        iter::once(&self.0)
    }
}

impl<const N: usize, T> sealed::Sealed for [T; N] {}
impl<const N: usize, T> Coll for [T; N] {
    type Item = T;
    type Map<U> = [U; N];
    type Iter<'a> = <&'a Self as IntoIterator>::IntoIter where T: 'a;

    const STATIC_SIZE: Option<NonZeroUsize> = NonZeroUsize::new(N);

    fn size(&self) -> usize {
        N
    }

    fn iter(&self) -> Self::Iter<'_> {
        <&Self as IntoIterator>::into_iter(self)
    }
}

impl<T> sealed::Sealed for Vec<T> {}
impl<T> Coll for Vec<T> {
    type Item = T;
    type Map<U> = Vec<U>;
    type Iter<'a> = <&'a Self as IntoIterator>::IntoIter where T: 'a;

    const STATIC_SIZE: Option<NonZeroUsize> = None;

    fn size(&self) -> usize {
        self.len()
    }

    fn iter(&self) -> Self::Iter<'_> {
        <&Self as IntoIterator>::into_iter(self)
    }
}
