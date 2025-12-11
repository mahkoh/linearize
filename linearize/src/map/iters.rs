#[allow(unused_imports)]
use crate::StaticMap;
use {
    crate::Linearize,
    core::{iter::Enumerate, marker::PhantomData, ops::Range},
};

/// An immutable iterator over the keys and values of a [`StaticMap`].
pub struct Iter<'a, L, T>
where
    L: Linearize,
{
    iter: Range<usize>,
    storage: *const T,
    _phantom: PhantomData<fn() -> (L, &'a T)>,
}

impl<'a, L, T> Iter<'a, L, T>
where
    L: Linearize,
    T: 'a,
{
    pub(super) fn new(storage: &'a L::Storage<T>) -> Self {
        Self {
            iter: 0..L::LENGTH,
            storage: <L::Storage<T> as AsRef<[T]>>::as_ref(storage).as_ptr(),
            _phantom: Default::default(),
        }
    }

    /// # Safety
    ///
    /// i must have been returned by self.iter
    unsafe fn item(storage: *const T, i: usize) -> (L, &'a T) {
        // SAFETY: self.iter only returns values in 0..L::LENGTH.
        let k = L::from_linear_unchecked(i);
        // SAFETY:
        // - *self.storage is L::Storage<T>
        // - L::Storage<T> is required to be [T; L::LENGTH]
        // - [T; L::LENGTH]: AsRef<[T]> returns a slice of length L::LENGTH;
        // - self.storage is a pointer to the first element of this slice
        // - i is less than L::LENGTH
        let v = &*storage.add(i);
        (k, v)
    }
}

impl<'a, L, T> Clone for Iter<'a, L, T>
where
    L: Linearize,
    T: 'a,
{
    fn clone(&self) -> Self {
        Self {
            iter: self.iter.clone(),
            storage: self.storage,
            _phantom: Default::default(),
        }
    }
}

macro_rules! impl_iter {
    ($name:ident, $ref_type:ty) => {
        impl<'a, L, T> Iterator for $name<'a, L, T>
        where
            L: Linearize,
            T: 'a,
        {
            type Item = (L, $ref_type);

            fn next(&mut self) -> Option<Self::Item> {
                self.iter.next().map(|i| unsafe {
                    // SAFETY: i was returned by self.iter
                    Self::item(self.storage, i)
                })
            }

            fn size_hint(&self) -> (usize, Option<usize>) {
                self.iter.size_hint()
            }

            fn count(self) -> usize
            where
                Self: Sized,
            {
                self.iter.count()
            }

            fn last(self) -> Option<Self::Item>
            where
                Self: Sized,
            {
                self.iter.last().map(|i| unsafe {
                    // SAFETY: i was returned by self.iter
                    Self::item(self.storage, i)
                })
            }

            fn nth(&mut self, n: usize) -> Option<Self::Item> {
                self.iter.nth(n).map(|i| unsafe {
                    // SAFETY: i was returned by self.iter
                    Self::item(self.storage, i)
                })
            }
        }

        impl<'a, L, T> ExactSizeIterator for $name<'a, L, T>
        where
            L: Linearize,
            T: 'a,
        {
        }

        impl<'a, L, T> DoubleEndedIterator for $name<'a, L, T>
        where
            L: Linearize,
            T: 'a,
        {
            fn next_back(&mut self) -> Option<Self::Item> {
                self.iter.next_back().map(|i| unsafe {
                    // SAFETY: i was returned by self.iter
                    Self::item(self.storage, i)
                })
            }

            fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
                self.iter.nth_back(n).map(|i| unsafe {
                    // SAFETY: i was returned by self.iter
                    Self::item(self.storage, i)
                })
            }
        }
    };
}

impl_iter!(Iter, &'a T);

/// A mutable iterator over the keys and values of a [`StaticMap`].
pub struct IterMut<'a, L, T>
where
    L: Linearize,
{
    iter: Range<usize>,
    storage: *mut T,
    _phantom: PhantomData<fn() -> (L, &'a mut T)>,
}

impl<'a, L, T> IterMut<'a, L, T>
where
    L: Linearize,
    T: 'a,
{
    pub(super) fn new(storage: &'a mut L::Storage<T>) -> Self {
        Self {
            iter: 0..L::LENGTH,
            storage: <L::Storage<T> as AsMut<[T]>>::as_mut(storage).as_mut_ptr(),
            _phantom: Default::default(),
        }
    }

    /// # Safety
    ///
    /// - i must have been returned by self.iter
    /// - no i must be used more than once
    unsafe fn item(storage: *mut T, i: usize) -> (L, &'a mut T) {
        // SAFETY: self.iter only returns values in 0..L::LENGTH.
        let k = L::from_linear_unchecked(i);
        // SAFETY:
        // - L::Storage<T> is required to be [T; L::LENGTH]
        // - [T; L::LENGTH]: AsMut<[T]> returns a slice of length L::LENGTH
        // - self.storage is a pointer to the first element of this slice
        // - i is less than L::LENGTH
        // - Each i appears at most once in this iterator
        let v = &mut *storage.add(i);
        (k, v)
    }
}

// SAFETY: We never clone self.iter so every value returned by it is unique.
impl_iter!(IterMut, &'a mut T);

/// A consuming iterator over the keys and values of a [`StaticMap`].
pub struct IntoIter<L, T>
where
    L: Linearize,
{
    iter: Enumerate<<<L as Linearize>::Storage<T> as IntoIterator>::IntoIter>,
}

impl<L, T> IntoIter<L, T>
where
    L: Linearize,
{
    pub(super) fn new(storage: L::Storage<T>) -> Self {
        Self {
            iter: <L::Storage<T> as IntoIterator>::into_iter(storage).enumerate(),
        }
    }

    /// # Safety
    ///
    /// - i must have been returned by self.iter
    unsafe fn key(i: usize) -> L {
        unsafe {
            // SAFETY:
            // - self.iter is <L::Storage<T> as IntoIterator>::into_iter(storage).enumerate().
            // - L::Storage<T> is [T; L::LENGTH].
            // - Therefore, <L::Storage<T> as IntoIterator>::into_iter(storage) returns
            //   exactly L::LENGTH elements.
            // - Therefore, i < L::LENGTH.
            L::from_linear_unchecked(i)
        }
    }
}

impl<L, T> Iterator for IntoIter<L, T>
where
    L: Linearize,
{
    type Item = (L, T);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(i, v)| {
            let k = unsafe {
                // SAFETY: i was returned by self.iter
                Self::key(i)
            };
            (k, v)
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.iter.count()
    }

    fn last(self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        #[expect(clippy::double_ended_iterator_last)]
        self.iter.last().map(|(i, v)| {
            let k = unsafe {
                // SAFETY: i was returned by self.iter
                Self::key(i)
            };
            (k, v)
        })
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.iter.nth(n).map(|(i, v)| {
            let k = unsafe {
                // SAFETY: i was returned by self.iter
                Self::key(i)
            };
            (k, v)
        })
    }
}

impl<L, T> ExactSizeIterator for IntoIter<L, T> where L: Linearize {}

impl<L, T> DoubleEndedIterator for IntoIter<L, T>
where
    L: Linearize,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(|(i, v)| {
            let k = unsafe {
                // SAFETY: i was returned by self.iter
                Self::key(i)
            };
            (k, v)
        })
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.iter.nth_back(n).map(|(i, v)| {
            let k = unsafe {
                // SAFETY: i was returned by self.iter
                Self::key(i)
            };
            (k, v)
        })
    }
}

/// A consuming iterator over the keys and values of a [`StaticMap`].
pub struct IntoValues<L, T>
where
    L: Linearize,
{
    iter: <<L as Linearize>::Storage<T> as IntoIterator>::IntoIter,
}

impl<L, T> IntoValues<L, T>
where
    L: Linearize,
{
    pub(super) fn new(storage: L::Storage<T>) -> Self {
        Self {
            iter: <L::Storage<T> as IntoIterator>::into_iter(storage),
        }
    }
}

impl<L, T> Iterator for IntoValues<L, T>
where
    L: Linearize,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.iter.count()
    }

    fn last(self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        self.iter.last()
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.iter.nth(n)
    }
}

impl<L, T> ExactSizeIterator for IntoValues<L, T> where L: Linearize {}

impl<L, T> DoubleEndedIterator for IntoValues<L, T>
where
    L: Linearize,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back()
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.iter.nth_back(n)
    }
}
