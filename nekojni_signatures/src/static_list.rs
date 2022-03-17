use std::{
    cmp::Ordering,
    hash::{Hash, Hasher},
    ops::Deref,
};

/// Represents a list that may or may not be borrowed from another list.
///
/// This is a refinement of `Cow<'a, [T]>` that avoids a lot of weird issues caused by lifetimes
/// and recursive data structures due the trait magic `Cow` uses.
#[derive(Debug, Clone)]
pub enum StaticList<'a, T: 'a> {
    /// A borrowed list.
    Borrowed(&'a [T]),
    /// An owned list.
    Owned(Vec<T>),
}
impl<'a, T: 'a> StaticList<'a, T> {
    /// Returns this as a borrowed slice.
    pub fn as_slice(&self) -> &[T] {
        match self {
            StaticList::Borrowed(arr) => arr,
            StaticList::Owned(vec) => &vec,
        }
    }
}
impl<'a, T: 'a + Clone> StaticList<'a, T> {
    /// Returns this as an owned `Vec`. This always clones the internal contents.
    pub fn to_owned(&self) -> Vec<T> {
        match self {
            StaticList::Borrowed(arr) => arr.to_vec(),
            StaticList::Owned(vec) => vec.clone(),
        }
    }

    /// Converts this into an owned `Vec`.
    pub fn into_owned(self) -> Vec<T> {
        match self {
            StaticList::Borrowed(arr) => arr.to_vec(),
            StaticList::Owned(vec) => vec,
        }
    }
}
impl<'a, T: 'a> From<&'a [T]> for StaticList<'a, T> {
    fn from(arr: &'a [T]) -> Self {
        StaticList::Borrowed(arr)
    }
}
impl<'a, T: 'a> From<Vec<T>> for StaticList<'a, T> {
    fn from(arr: Vec<T>) -> Self {
        StaticList::Owned(arr)
    }
}
impl<'a, T: 'a> Deref for StaticList<'a, T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        self.as_slice()
    }
}
impl<'a, T: 'a + Hash> Hash for StaticList<'a, T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Hash::hash(self.as_slice(), state)
    }
}
impl<'a, T: 'a + Ord> Ord for StaticList<'a, T> {
    fn cmp(&self, other: &Self) -> Ordering {
        Ord::cmp(self.as_slice(), other.as_slice())
    }
}
impl<'a, T: 'a + PartialOrd> PartialOrd for StaticList<'a, T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        PartialOrd::partial_cmp(self.as_slice(), other.as_slice())
    }
}
impl<'a, T: 'a + Eq> Eq for StaticList<'a, T> {}
impl<'a, T: 'a + PartialEq> PartialEq for StaticList<'a, T> {
    fn eq(&self, other: &Self) -> bool {
        PartialEq::eq(self.as_slice(), other.as_slice())
    }
}
