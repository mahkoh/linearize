use {
    crate::{
        copy_map::StaticCopyMap,
        map::iters::{IntoIter, IntoValues, Iter, IterMut},
        storage::Storage,
        variants::Variants,
        Linearize, LinearizeExt, Linearized,
    },
    core::{
        array::TryFromSliceError,
        borrow::{Borrow, BorrowMut},
        cmp::Ordering,
        fmt::{Debug, Formatter},
        hash::{Hash, Hasher},
        mem,
        ops::{Deref, DerefMut, Index, IndexMut},
    },
};

pub(crate) mod iters;

/// An array-backed map with complex keys.
///
/// This type is an optimized version of [HashMap](std::collections::HashMap).
///
/// # Example
///
/// ```rust
/// # use linearize::{StaticMap, Linearize};
/// #[derive(Linearize)]
/// enum Format {
///     R,
///     Rgb,
///     Rgba,
/// }
///
/// let mut channels = StaticMap::default();
/// channels[Format::R] = 1;
/// channels[Format::Rgb] = 3;
/// channels[Format::Rgba] = 4;
/// assert_eq!(channels[Format::R], 1);
/// ```
///
/// # Keys
///
/// The key type, `L`, must implement the [Linearize] trait. This trait is implemented for
/// a number of standard library types:
///
/// - `bool`
/// - `u8`
/// - `()`
/// - `Infallible`
/// - etc.
///
/// See the rustdoc for a full list of types.
///
/// You can implement this trait for your own types, either manually or by using the
/// [proc macro](linearize_derive::Linearize).
///
/// # Storage
///
/// A `StaticMap` is a transparent wrapper around `[T; L::LENGTH]`.
/// [LENGTH](Linearize::LENGTH) is an associated constant of the [Linearize] trait.
///
/// # Runtime Performance
///
/// Indexing into the map needs to compute the index into the array.
///
/// If the key type is an enum without fields, the index is the discriminant of the enum
/// variant, making this computation a no-op.
///
/// For complex types, such as enums with fields, this computation is not always free. If
/// you index with the same key multiple times, you can pre-compute the array index.
///
/// ```rust
/// # use linearize::{Linearize, LinearizeExt, StaticMap};
/// fn dynamic_access<L: Linearize>(map: &mut StaticMap<L, u8>, key: &L) {
///     map[key] = map[key] + 1;
/// }
///
/// fn pre_computed_access<L: Linearize>(map: &mut StaticMap<L, u8>, key: &L) {
///     let key = key.linearized();
///     map[key] = map[key] + 1;
/// }
/// ```
///
/// # Traits
///
/// `StaticMap` unconditionally implements the following traits by forwarding to the
/// underlying `[T; L::LENGTH]`:
///
/// - `AsMut<[T]>`
/// - `AsRef<[T]>`
/// - `Borrow<[T]>`
/// - `BorrowMut<[T]>`
/// - `Deref<Target=[T]>`
/// - `DerefMut`
/// - `TryFrom`
/// - `IntoIterator`
///
/// The following traits are implemented unconditionally and behave as they behave for a
/// map:
///
/// - `Extend`
/// - `FromIterator`
/// - `Index<L>`
/// - `Index<&L>`
/// - `IndexMut<L>`
/// - `IndexMut<&L>`
///
/// The following traits are implemented if `T` implements them by forwarding to the
/// underlying `[T; L::LENGTH]`:
///
/// - [`Arbitrary`](arbitrary_1::Arbitrary) if the `arbitrary-1` feature is enabled.
/// - `Clone`
/// - `Default`
/// - `Eq`
/// - `Hash`
/// - `Ord`
/// - `PartialEq`
/// - `PartialOrd`
///
/// The following traits are implemented if both `L` and `T` implement them and their
/// implementation behaves as they behave for a map:
///
/// - `Debug`
/// - [`Deserialize`](serde_1::Deserialize) if the `serde-1` feature is enabled.
/// - [`Serialize`](serde_1::Serialize) if the `serde-1` feature is enabled.
///
/// The `Deserialize` implementation requires that the input contains all possible keys.
/// You can modify this behavior by using the tools in the [serde_1](crate::serde_1)
/// module.
///
/// If the `rand-0_8` feature is enabled, the
/// [`Distribution<StaticMap<L, T>>`](rand_0_8::distributions::Distribution) trait is
/// implemented for the following distributions by forwarding to the underlying
/// `[T; L::LENGTH]`:
///
/// - [`Bernoulli`](rand_0_8::distributions::Bernoulli)
/// - [`Open01`](rand_0_8::distributions::Open01)
/// - [`OpenClosed01`](rand_0_8::distributions::OpenClosed01)
/// - [`Standard`](rand_0_8::distributions::Standard)
/// - [`Uniform`](rand_0_8::distributions::Uniform)
/// - [`WeightedIndex`](rand_0_8::distributions::WeightedIndex)
///
/// If the `rand-0_9` feature is enabled, the
/// [`Distribution<StaticMap<L, T>>`](rand_0_9::distr::Distribution) trait is
/// implemented for the following distributions by forwarding to the underlying
/// `[T; L::LENGTH]`:
///
/// - [`Bernoulli`](rand_0_9::distr::Bernoulli)
/// - [`Open01`](rand_0_9::distr::Open01)
/// - [`OpenClosed01`](rand_0_9::distr::OpenClosed01)
/// - [`StandardUniform`](rand_0_9::distr::StandardUniform)
/// - [`Uniform`](rand_0_9::distr::Uniform)
/// - [`WeightedIndex`](rand_0_9::distr::weighted::WeightedIndex)
///
/// If the `rand-0_10` feature is enabled, the
/// [`Distribution<StaticMap<L, T>>`](rand_0_10::distr::Distribution) trait is
/// implemented for the following distributions by forwarding to the underlying
/// `[T; L::LENGTH]`:
///
/// - [`Bernoulli`](rand_0_10::distr::Bernoulli)
/// - [`Open01`](rand_0_10::distr::Open01)
/// - [`OpenClosed01`](rand_0_10::distr::OpenClosed01)
/// - [`StandardUniform`](rand_0_10::distr::StandardUniform)
/// - [`Uniform`](rand_0_10::distr::Uniform)
/// - [`WeightedIndex`](rand_0_10::distr::weighted::WeightedIndex)
///
/// # Copy Trait
///
/// This type **never** implements `Copy`. This is due to a limitation of the rust type
/// system.
///
/// Use [`StaticCopyMap`] if `T` is `Copy` and you want the map to be `Copy` as well.
#[repr(transparent)]
pub struct StaticMap<L, T>(
    /// The underlying `[T; L::LENGTH]`.
    pub <L as Linearize>::Storage<T>,
)
where
    L: Linearize + ?Sized;

impl<L, T> StaticMap<L, T>
where
    L: Linearize + ?Sized,
{
    /// Creates a map from a callback.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use linearize::StaticMap;
    /// let map = StaticMap::from_fn(|l: bool| l as u32);
    /// ```
    #[inline]
    pub fn from_fn(mut cb: impl FnMut(L) -> T) -> Self
    where
        L: Sized,
    {
        Self(<L::Storage<T> as Storage<L, T>>::from_fn(|i| unsafe {
            // SAFETY:
            // - Storage::from_fn has only one implementation implementation and it calls
            //   core::array::from_fn::<[T; L::LENGTH]>.
            // - Therefore i is less that L::LENGTH.
            cb(L::from_linear_unchecked(i))
        }))
    }

    /// Creates a map from a reference to the underlying storage.
    ///
    /// Due to limitations of the rust type system, the underlying type is opaque in code
    /// that is generic over `L`. However, in code with concrete `L`s, this function can
    /// be used to turn any `[T; L::LENGTH]` into a map.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use linearize::StaticMap;
    /// let array = [0, 1];
    /// let map = StaticMap::from_ref(&array);
    /// assert_eq!(map[false], 0);
    /// assert_eq!(map[true], 1);
    /// ```
    #[inline]
    pub const fn from_ref(storage: &<L as Linearize>::Storage<T>) -> &Self {
        unsafe {
            // SAFETY: Self is a transparent wrapper around L::Storage<T>.
            mem::transmute(storage)
        }
    }

    /// Creates a map from a mutable reference to the underlying storage.
    ///
    /// Due to limitations of the rust type system, the underlying type is opaque in code
    /// that is generic over `L`. However, in code with concrete `L`s, this function can
    /// be used to turn any `[T; L::LENGTH]` into a map.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use linearize::StaticMap;
    /// let mut array = [0, 1];
    /// let map = StaticMap::from_mut(&mut array);
    /// map[false] = 1;
    /// map[true] = 0;
    /// assert_eq!(array, [1, 0]);
    /// ```
    #[inline]
    pub fn from_mut(storage: &mut <L as Linearize>::Storage<T>) -> &mut Self {
        unsafe {
            // SAFETY: Self is a transparent wrapper around L::Storage<T>.
            mem::transmute(storage)
        }
    }

    /// Fallibly collects an iterator into a map. Returns Err if some key is not produced by
    /// the iterator.
    ///
    /// # Example
    ///
    /// ```rust
    /// use linearize::StaticMap;
    /// let map = StaticMap::try_from_iter([(false, 0), (true, 1)]);
    /// assert_eq!(map, Ok(StaticMap([0, 1])));
    ///
    /// let map = StaticMap::try_from_iter([(false, 0)]);
    /// assert_eq!(map, Err(StaticMap([Some(0), None])));
    /// ```
    #[inline]
    pub fn try_from_iter(
        iter: impl IntoIterator<Item = (L, T)>,
    ) -> Result<Self, StaticMap<L, Option<T>>>
    where
        L: Sized,
    {
        let mut res = StaticMap::<L, Option<T>>::default();
        for (k, v) in iter {
            res[k] = Some(v);
        }
        if res.values().any(|v| v.is_none()) {
            return Err(res);
        }
        Ok(res.map_values(|v| v.unwrap()))
    }

    /// Converts this map to a [StaticCopyMap].
    ///
    /// This is a zero-cost conversion.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use linearize::{static_map, StaticCopyMap, StaticMap};
    /// let map: StaticMap<_, _> = static_map! {
    ///     false => 0,
    ///     true => 1,
    /// };
    /// let map: StaticCopyMap<_, _> = map.into_copy();
    /// assert_eq!(map[false], 0);
    /// assert_eq!(map[true], 1);
    /// ```
    #[inline]
    pub fn into_copy(self) -> StaticCopyMap<L, T>
    where
        T: Copy,
    {
        StaticCopyMap(self.0.into_copy())
    }

    /// Converts a [StaticCopyMap] to a [StaticMap].
    ///
    /// This is a zero-cost conversion.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use linearize::{static_copy_map, StaticCopyMap, StaticMap};
    /// let map: StaticCopyMap<_, _> = static_copy_map! {
    ///     false => 0,
    ///     true => 1,
    /// };
    /// let map: StaticMap<_, _> = StaticMap::from_copy(map);
    /// assert_eq!(map[false], 0);
    /// assert_eq!(map[true], 1);
    /// ```
    #[inline]
    pub fn from_copy(map: StaticCopyMap<L, T>) -> Self
    where
        T: Copy,
    {
        map.into_static_map()
    }

    /// Converts a reference to this map to a reference to a [StaticCopyMap].
    ///
    /// This is a zero-cost re-interpretation conversion.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use linearize::{static_map, StaticCopyMap, StaticMap};
    /// let map: StaticMap<_, _> = static_map! {
    ///     false => 0,
    ///     true => 1,
    /// };
    /// let map: &StaticCopyMap<_, _> = map.as_copy();
    /// assert_eq!(map[false], 0);
    /// assert_eq!(map[true], 1);
    /// ```
    #[inline]
    pub fn as_copy(&self) -> &StaticCopyMap<L, T>
    where
        T: Copy,
    {
        StaticCopyMap::from_ref(self.0.as_copy())
    }

    /// Converts a mutable reference to this map to a mutable reference to a [StaticCopyMap].
    ///
    /// This is a zero-cost re-interpretation conversion.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use linearize::{static_map, StaticCopyMap, StaticMap};
    /// let mut map: StaticMap<_, _> = static_map! {
    ///     false => 0,
    ///     true => 1,
    /// };
    /// assert_eq!(map[false], 0);
    /// assert_eq!(map[true], 1);
    /// {
    ///     let map: &mut StaticCopyMap<_, _> = map.as_copy_mut();
    ///     map[false] = 1;
    ///     map[true] = 0;
    /// }
    /// assert_eq!(map[false], 1);
    /// assert_eq!(map[true], 0);
    /// ```
    #[inline]
    pub fn as_copy_mut(&mut self) -> &mut StaticCopyMap<L, T>
    where
        T: Copy,
    {
        StaticCopyMap::from_mut(self.0.as_copy_mut())
    }

    /// Creates a new map whose values are references to the values in this map.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use linearize::{static_map, StaticCopyMap, StaticMap};
    /// let map: StaticMap<_, u8> = static_map! {
    ///     false => 0,
    ///     true => 1,
    /// };
    /// let map: StaticCopyMap<_, &u8> = map.each_ref();
    /// assert_eq!(map[false], &0);
    /// assert_eq!(map[true], &1);
    /// ```
    #[inline]
    pub fn each_ref(&self) -> StaticCopyMap<L, &T> {
        StaticCopyMap(self.0.each_ref().into_copy())
    }

    /// Creates a new map whose values are mutable references to the values in this map.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use linearize::{static_map, StaticCopyMap, StaticMap};
    /// let mut map: StaticMap<_, u8> = static_map! {
    ///     false => 0,
    ///     true => 1,
    /// };
    /// assert_eq!(map[false], 0);
    /// assert_eq!(map[true], 1);
    /// {
    ///     let mut map: StaticMap<_, &mut u8> = map.each_mut();
    ///     *map[false] = 1;
    ///     *map[true] = 0;
    /// }
    /// assert_eq!(map[false], 1);
    /// assert_eq!(map[true], 0);
    /// ```
    #[inline]
    pub fn each_mut(&mut self) -> StaticMap<L, &mut T> {
        StaticMap(self.0.each_mut())
    }

    /// Remaps the values of this type.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use linearize::{static_map, StaticCopyMap, StaticMap};
    /// let map: StaticMap<_, u8> = static_map! {
    ///     false => 0,
    ///     true => 1,
    /// };
    /// assert_eq!(map[false], 0);
    /// assert_eq!(map[true], 1);
    /// let map = map.map(|b, v| b as u8 + v);
    /// assert_eq!(map[false], 0);
    /// assert_eq!(map[true], 2);
    /// ```
    #[inline]
    pub fn map<U>(self, mut map: impl FnMut(L, T) -> U) -> StaticMap<L, U>
    where
        L: Sized,
    {
        StaticMap(self.0.map(|i, t| {
            let l = unsafe {
                // SAFETY:
                // - L::Storage<T> is required to be [T; L::LENGTH].
                // - The implementation of Storage<T> for [T; N] calls this callback only
                //   with i < N.
                // - Therefore i < N = L::LENGTH.
                L::from_linear_unchecked(i)
            };
            map(l, t)
        }))
    }

    /// Remaps the values of this type without retrieving the keys.
    ///
    /// If you don't need access to the keys, this function can be more efficient.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use linearize::{static_map, StaticCopyMap, StaticMap};
    /// let map: StaticMap<_, u8> = static_map! {
    ///     false => 0,
    ///     true => 1,
    /// };
    /// assert_eq!(map[false], 0);
    /// assert_eq!(map[true], 1);
    /// let map = map.map_values(|v| 3 * v);
    /// assert_eq!(map[false], 0);
    /// assert_eq!(map[true], 3);
    /// ```
    #[inline]
    pub fn map_values<U>(self, mut map: impl FnMut(T) -> U) -> StaticMap<L, U> {
        StaticMap(self.0.map(|_, t| map(t)))
    }

    /// Resets all values in this map to their defaults.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use linearize::{static_map, StaticCopyMap, StaticMap};
    /// let mut map: StaticMap<_, u8> = static_map! {
    ///     false => 0,
    ///     true => 1,
    /// };
    /// assert_eq!(map[false], 0);
    /// assert_eq!(map[true], 1);
    /// map.clear();
    /// assert_eq!(map[false], 0);
    /// assert_eq!(map[true], 0);
    /// ```
    #[inline]
    pub fn clear(&mut self)
    where
        T: Default,
    {
        for v in self.0.as_mut() {
            *v = Default::default();
        }
    }

    /// Returns an iterator over the keys in this map.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use linearize::{static_map, StaticCopyMap, StaticMap};
    /// let map: StaticMap<_, u8> = static_map! {
    ///     false => 0,
    ///     true => 1,
    /// };
    /// let mut iter = map.keys();
    /// assert_eq!(iter.next(), Some(false));
    /// assert_eq!(iter.next(), Some(true));
    /// assert_eq!(iter.next(), None);
    /// ```
    #[inline]
    pub fn keys(&self) -> Variants<L>
    where
        L: Sized,
    {
        L::variants()
    }

    /// Returns an iterator over references to the values in this map.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use linearize::{static_map, StaticCopyMap, StaticMap};
    /// let map: StaticMap<_, u8> = static_map! {
    ///     false => 0,
    ///     true => 1,
    /// };
    /// let mut iter = map.values();
    /// assert_eq!(iter.next(), Some(&0));
    /// assert_eq!(iter.next(), Some(&1));
    /// assert_eq!(iter.next(), None);
    /// ```
    #[inline]
    pub fn values(&self) -> core::slice::Iter<'_, T> {
        self.as_ref().iter()
    }

    /// Returns an iterator over mutable references to the values in this map.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use linearize::{static_map, StaticCopyMap, StaticMap};
    /// let mut map: StaticMap<_, u8> = static_map! {
    ///     false => 0,
    ///     true => 1,
    /// };
    /// let mut iter = map.values_mut();
    /// assert_eq!(iter.next(), Some(&mut 0));
    /// assert_eq!(iter.next(), Some(&mut 1));
    /// assert_eq!(iter.next(), None);
    /// ```
    #[inline]
    pub fn values_mut(&mut self) -> core::slice::IterMut<'_, T> {
        self.as_mut().iter_mut()
    }

    /// Returns an iterator over references to the entries in this map.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use linearize::{static_map, StaticCopyMap, StaticMap};
    /// let map: StaticMap<_, u8> = static_map! {
    ///     false => 0,
    ///     true => 1,
    /// };
    /// let mut iter = map.iter();
    /// assert_eq!(iter.next(), Some((false, &0)));
    /// assert_eq!(iter.next(), Some((true, &1)));
    /// assert_eq!(iter.next(), None);
    /// ```
    #[inline]
    pub fn iter(&self) -> Iter<'_, L, T>
    where
        L: Sized,
    {
        Iter::new(&self.0)
    }

    /// Returns an iterator over mutable references to the entries in this map.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use linearize::{static_map, StaticCopyMap, StaticMap};
    /// let mut map: StaticMap<_, u8> = static_map! {
    ///     false => 0,
    ///     true => 1,
    /// };
    /// let mut iter = map.iter_mut();
    /// assert_eq!(iter.next(), Some((false, &mut 0)));
    /// assert_eq!(iter.next(), Some((true, &mut 1)));
    /// assert_eq!(iter.next(), None);
    /// ```
    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_, L, T>
    where
        L: Sized,
    {
        IterMut::new(&mut self.0)
    }

    /// Consumes the map and returns an iterator over the values.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use linearize::{static_map, StaticMap};
    /// let mut map: StaticMap<_, u8> = static_map! {
    ///     false => 0,
    ///     true => 1,
    /// };
    /// let mut iter = map.into_values();
    /// assert_eq!(iter.next(), Some(0));
    /// assert_eq!(iter.next(), Some(1));
    /// assert_eq!(iter.next(), None);
    /// ```   
    pub fn into_values(self) -> IntoValues<L, T>
    where
        L: Sized,
    {
        IntoValues::new(self.0)
    }
}

impl<L, T> Deref for StaticMap<L, T>
where
    L: Linearize + ?Sized,
{
    type Target = [T];

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

impl<L, T> DerefMut for StaticMap<L, T>
where
    L: Linearize + ?Sized,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut()
    }
}

impl<L, T> FromIterator<(L, T)> for StaticMap<L, T>
where
    L: Linearize,
    T: Default,
{
    #[inline]
    fn from_iter<I: IntoIterator<Item = (L, T)>>(iter: I) -> Self {
        let mut res = StaticMap::<L, Option<T>>::default();
        for (k, v) in iter {
            res[k] = Some(v);
        }
        res.map_values(|v| v.unwrap_or_default())
    }
}

impl<'a, L, T> FromIterator<(&'a L, T)> for StaticMap<L, T>
where
    L: Linearize,
    T: Default,
{
    #[inline]
    fn from_iter<I: IntoIterator<Item = (&'a L, T)>>(iter: I) -> Self {
        let mut res = StaticMap::<L, Option<T>>::default();
        for (k, v) in iter {
            res[k] = Some(v);
        }
        res.map_values(|v| v.unwrap_or_default())
    }
}

impl<L, T> Clone for StaticMap<L, T>
where
    L: Linearize + ?Sized,
    T: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        Self(<L::Storage<T> as Storage<L, T>>::clone(&self.0))
    }

    #[inline]
    fn clone_from(&mut self, source: &Self) {
        <L::Storage<T> as Storage<L, T>>::clone_from(&mut self.0, &source.0);
    }
}

impl<L, T> Index<&'_ L> for StaticMap<L, T>
where
    L: Linearize + ?Sized,
{
    type Output = T;

    #[inline]
    fn index(&self, index: &L) -> &Self::Output {
        self.index(index.linearized())
    }
}

impl<L, T> Index<L> for StaticMap<L, T>
where
    L: Linearize,
{
    type Output = T;

    #[inline]
    fn index(&self, index: L) -> &Self::Output {
        self.index(index.linearized())
    }
}

impl<L, T> Index<Linearized<L>> for StaticMap<L, T>
where
    L: Linearize + ?Sized,
{
    type Output = T;

    #[inline(always)]
    fn index(&self, index: Linearized<L>) -> &Self::Output {
        unsafe {
            // SAFETY:
            // - self.0 is L::Storage which is required to be [T; L::LENGTH].
            // - [T; L::LENGTH]: AsRef<[T]> returns &[T] with length L::LENGTH.
            // - Key::<L>::get returns a value less than L::LENGTH.
            <L::Storage<T> as AsRef<[T]>>::as_ref(&self.0).get_unchecked(index.get())
        }
    }
}

impl<L, T> IndexMut<&'_ L> for StaticMap<L, T>
where
    L: Linearize + ?Sized,
{
    #[inline]
    fn index_mut(&mut self, index: &L) -> &mut Self::Output {
        self.index_mut(index.linearized())
    }
}

impl<L, T> IndexMut<L> for StaticMap<L, T>
where
    L: Linearize,
{
    #[inline]
    fn index_mut(&mut self, index: L) -> &mut Self::Output {
        self.index_mut(index.linearized())
    }
}

impl<L, T> IndexMut<Linearized<L>> for StaticMap<L, T>
where
    L: Linearize + ?Sized,
{
    #[inline(always)]
    fn index_mut(&mut self, index: Linearized<L>) -> &mut Self::Output {
        unsafe {
            // SAFETY:
            // - self.0 is L::Storage which is required to be [T; L::LENGTH].
            // - [T; L::LENGTH]: AsMut<[T]> returns &mut [T] with length L::LENGTH.
            // - Key::<L>::get returns a value less than L::LENGTH.
            <L::Storage<T> as AsMut<[T]>>::as_mut(&mut self.0).get_unchecked_mut(index.get())
        }
    }
}

impl<L, T> AsMut<[T]> for StaticMap<L, T>
where
    L: Linearize + ?Sized,
{
    #[inline]
    fn as_mut(&mut self) -> &mut [T] {
        self.0.as_mut()
    }
}

impl<L, T> AsRef<[T]> for StaticMap<L, T>
where
    L: Linearize + ?Sized,
{
    #[inline]
    fn as_ref(&self) -> &[T] {
        self.0.as_ref()
    }
}

impl<L, T> Borrow<[T]> for StaticMap<L, T>
where
    L: Linearize + ?Sized,
{
    #[inline]
    fn borrow(&self) -> &[T] {
        self.0.borrow()
    }
}

impl<L, T> BorrowMut<[T]> for StaticMap<L, T>
where
    L: Linearize + ?Sized,
{
    #[inline]
    fn borrow_mut(&mut self) -> &mut [T] {
        self.0.borrow_mut()
    }
}

impl<L, T> Debug for StaticMap<L, T>
where
    L: Linearize,
    L: Debug,
    T: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let mut map = f.debug_map();
        for (k, v) in self {
            map.key(&k).value(v);
        }
        map.finish()
    }
}

impl<L, T> Default for StaticMap<L, T>
where
    L: Linearize + ?Sized,
    T: Default,
{
    #[inline]
    fn default() -> Self {
        Self(Storage::default())
    }
}

impl<L, T> Eq for StaticMap<L, T>
where
    L: Linearize + ?Sized,
    T: Eq,
{
}

impl<L, T> Hash for StaticMap<L, T>
where
    L: Linearize + ?Sized,
    T: Hash,
{
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.as_hash().hash(state)
    }
}

impl<L, T> Ord for StaticMap<L, T>
where
    L: Linearize + ?Sized,
    T: Ord,
{
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }

    #[inline]
    fn max(self, other: Self) -> Self
    where
        Self: Sized,
    {
        Self(self.0.max(other.0))
    }

    #[inline]
    fn min(self, other: Self) -> Self
    where
        Self: Sized,
    {
        Self(self.0.min(other.0))
    }

    #[inline]
    fn clamp(self, min: Self, max: Self) -> Self
    where
        Self: Sized,
        Self: PartialOrd,
    {
        Self(self.0.clamp(min.0, max.0))
    }
}

impl<L, T> PartialEq for StaticMap<L, T>
where
    L: Linearize + ?Sized,
    T: PartialEq,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl<L, T> PartialOrd for StaticMap<L, T>
where
    L: Linearize + ?Sized,
    T: PartialOrd,
{
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<'a, L, T> TryFrom<&'a [T]> for &'a StaticMap<L, T>
where
    L: Linearize + ?Sized,
{
    type Error = TryFromSliceError;

    #[inline]
    fn try_from(value: &'a [T]) -> Result<Self, Self::Error> {
        <L::Storage<T>>::ref_try_from(value).map(StaticMap::from_ref)
    }
}

impl<'a, L, T> TryFrom<&'a mut [T]> for &'a mut StaticMap<L, T>
where
    L: Linearize + ?Sized,
    T:,
{
    type Error = TryFromSliceError;

    #[inline]
    fn try_from(value: &'a mut [T]) -> Result<Self, Self::Error> {
        <L::Storage<T>>::mut_try_from(value).map(StaticMap::from_mut)
    }
}

impl<L, T> TryFrom<&[T]> for StaticMap<L, T>
where
    L: Linearize + ?Sized,
    T: Copy,
{
    type Error = TryFromSliceError;

    #[inline]
    fn try_from(value: &[T]) -> Result<Self, Self::Error> {
        <L::Storage<T>>::copy_ref_try_from(value).map(Self)
    }
}

impl<L, T> TryFrom<&mut [T]> for StaticMap<L, T>
where
    L: Linearize + ?Sized,
    T: Copy,
{
    type Error = TryFromSliceError;

    #[inline]
    fn try_from(value: &mut [T]) -> Result<Self, Self::Error> {
        <L::Storage<T>>::copy_mut_try_from(value).map(Self)
    }
}

#[cfg(feature = "std")]
impl<L, T> TryFrom<Vec<T>> for StaticMap<L, T>
where
    L: Linearize + ?Sized,
{
    type Error = Vec<T>;

    #[inline]
    fn try_from(value: Vec<T>) -> Result<Self, Self::Error> {
        <L::Storage<T>>::vec_try_from(value).map(Self)
    }
}

impl<L, T> Extend<(L, T)> for StaticMap<L, T>
where
    L: Linearize,
{
    #[inline]
    fn extend<I: IntoIterator<Item = (L, T)>>(&mut self, iter: I) {
        for (k, v) in iter {
            self[k] = v;
        }
    }
}

impl<'a, L, T> Extend<(&'a L, &'a T)> for StaticMap<L, T>
where
    L: Linearize + ?Sized,
    T: Clone,
{
    #[inline]
    fn extend<I: IntoIterator<Item = (&'a L, &'a T)>>(&mut self, iter: I) {
        for (k, v) in iter {
            self[k] = v.clone();
        }
    }
}

impl<'a, L, T> IntoIterator for &'a StaticMap<L, T>
where
    L: Linearize,
    T: 'a,
{
    type Item = (L, &'a T);
    type IntoIter = Iter<'a, L, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, L, T> IntoIterator for &'a mut StaticMap<L, T>
where
    L: Linearize,
{
    type Item = (L, &'a mut T);
    type IntoIter = IterMut<'a, L, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<L, T> IntoIterator for StaticMap<L, T>
where
    L: Linearize,
{
    type Item = (L, T);
    type IntoIter = IntoIter<L, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        IntoIter::new(self.0.into_storage())
    }
}

impl<L, T> From<StaticCopyMap<L, T>> for StaticMap<L, T>
where
    L: Linearize,
    T: Copy,
{
    fn from(value: StaticCopyMap<L, T>) -> Self {
        value.into_static_map()
    }
}

impl<L, T> AsRef<StaticCopyMap<L, T>> for StaticMap<L, T>
where
    L: Linearize,
    T: Copy,
{
    fn as_ref(&self) -> &StaticCopyMap<L, T> {
        self.as_copy()
    }
}

impl<L, T> AsMut<StaticCopyMap<L, T>> for StaticMap<L, T>
where
    L: Linearize,
    T: Copy,
{
    fn as_mut(&mut self) -> &mut StaticCopyMap<L, T> {
        self.as_copy_mut()
    }
}

impl<L, T> Borrow<StaticCopyMap<L, T>> for StaticMap<L, T>
where
    L: Linearize,
    T: Copy,
{
    fn borrow(&self) -> &StaticCopyMap<L, T> {
        self.as_copy()
    }
}

impl<L, T> BorrowMut<StaticCopyMap<L, T>> for StaticMap<L, T>
where
    L: Linearize,
    T: Copy,
{
    fn borrow_mut(&mut self) -> &mut StaticCopyMap<L, T> {
        self.as_copy_mut()
    }
}
