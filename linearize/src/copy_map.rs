use {
    crate::{
        iter::{Iter, IterMut},
        map::iters::{IntoIter, IntoValues},
        storage::CopyStorage,
        Linearize, Linearized, StaticMap,
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

/// A copyable, array-backed map with complex keys.
///
/// This type is identical to [StaticMap] except that it always implements `Copy` and
/// requires the values to implement `Copy`. This type exists due to limitations of the
/// rust type system. It will be removed in a future version of this crate.
#[repr(transparent)]
pub struct StaticCopyMap<L, T>(
    /// The underlying `[T; L::LENGTH]`.
    pub <L as Linearize>::CopyStorage<T>,
)
where
    L: Linearize + ?Sized,
    T: Copy;

/// This custom implementation is guaranteed to always match the exact compiler generated
/// [Send] implementation thanks to the safety requirement
/// that `<L as Linearize>::CopyStorage<T>` is always just `[T;N]` for some `N`.
///
/// But in some generic contexts (when generic over `L` but with `T : Send`)
/// the compiler cannot tell that `<L as Linearize>::CopyStorage<T> : Send` because it
/// doesn't know that `<L as Linearize>::CopyStorage<T>` must be `[T;N]`.
///
/// This is necessary for the exact same reason why [StaticCopyMap] exists,
/// but no one wants a `StaticSendMap` or worse `StaticSendSyncCopyMap`,
/// and thankfully this can be avoided thanks to this manual implementation.
unsafe impl<L, T> Send for StaticCopyMap<L, T>
where
    L: Linearize + ?Sized,
    T: Send + Copy,
{
}

/// This custom implementation is guaranteed to always match the exact compiler generated
/// [Sync] implementation thanks to the safety requirement
/// that `<L as Linearize>::CopyStorage<T>` is always just `[T;N]` for some `N`.
///
/// But in some generic contexts (when generic over `L` but with `T : Sync`)
/// the compiler cannot tell that `<L as Linearize>::CopyStorage<T> : Sync` because it
/// doesn't know that `<L as Linearize>::CopyStorage<T>` must be `[T;N]`.
///
/// This is necessary for the exact same reason why [StaticCopyMap] exists,
/// but no one wants a `StaticSyncMap` or worse `StaticSendSyncCopyMap`,
/// and thankfully this can be avoided thanks to this manual implementation.
unsafe impl<L, T> Sync for StaticCopyMap<L, T>
where
    L: Linearize + ?Sized,
    T: Sync + Copy,
{
}

impl<L, T> StaticCopyMap<L, T>
where
    L: Linearize + ?Sized,
    T: Copy,
{
    /// Creates a map from a callback.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use linearize::StaticCopyMap;
    /// let map = StaticCopyMap::from_fn(|l: bool| l as u32);
    /// ```
    #[inline]
    pub fn from_fn(cb: impl FnMut(L) -> T) -> Self
    where
        L: Sized,
    {
        StaticMap::<L, T>::from_fn(cb).into_copy()
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
    /// # use linearize::StaticCopyMap;
    /// let array = [0, 1];
    /// let map = StaticCopyMap::from_ref(&array);
    /// assert_eq!(map[false], 0);
    /// assert_eq!(map[true], 1);
    /// ```
    #[inline]
    pub const fn from_ref(storage: &L::CopyStorage<T>) -> &Self {
        unsafe {
            // SAFETY: Self is a transparent wrapper around L::CopyStorage<T>.
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
    /// # use linearize::StaticCopyMap;
    /// let mut array = [0, 1];
    /// let map = StaticCopyMap::from_mut(&mut array);
    /// map[false] = 1;
    /// map[true] = 0;
    /// assert_eq!(array, [1, 0]);
    /// ```
    #[inline]
    pub fn from_mut(storage: &mut L::CopyStorage<T>) -> &mut Self {
        unsafe {
            // SAFETY: Self is a transparent wrapper around L::CopyStorage<T>.
            mem::transmute(storage)
        }
    }

    /// Fallibly collects an iterator into a map. Returns Err if some key is not produced by
    /// the iterator.
    ///
    /// # Example
    ///
    /// ```rust
    /// use linearize::StaticCopyMap;
    /// let map = StaticCopyMap::try_from_iter([(false, 0), (true, 1)]);
    /// assert_eq!(map, Ok(StaticCopyMap([0, 1])));
    ///
    /// let map = StaticCopyMap::try_from_iter([(false, 0)]);
    /// assert_eq!(map, Err(StaticCopyMap([Some(0), None])));
    /// ```
    #[inline]
    pub fn try_from_iter(
        iter: impl IntoIterator<Item = (L, T)>,
    ) -> Result<Self, StaticCopyMap<L, Option<T>>>
    where
        L: Sized,
    {
        match StaticMap::try_from_iter(iter) {
            Ok(map) => Ok(map.into_copy()),
            Err(map) => Err(map.into_copy()),
        }
    }

    /// Converts this map to a [StaticMap].
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
    /// let map: StaticMap<_, _> = map.into_static_map();
    /// assert_eq!(map[false], 0);
    /// assert_eq!(map[true], 1);
    /// ```
    #[inline]
    pub fn into_static_map(self) -> StaticMap<L, T> {
        StaticMap(self.0.into_storage())
    }

    /// Converts a [StaticMap] to a [StaticCopyMap].
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
    /// let map: StaticCopyMap<_, _> = StaticCopyMap::from_static_map(map);
    /// assert_eq!(map[false], 0);
    /// assert_eq!(map[true], 1);
    /// ```
    #[inline]
    pub fn from_static_map(map: StaticMap<L, T>) -> Self {
        map.into_copy()
    }

    /// Converts a reference to this map to a reference to a [StaticMap].
    ///
    /// This is a zero-cost re-interpretation conversion.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use linearize::{static_copy_map, StaticCopyMap, StaticMap};
    /// let map: StaticCopyMap<_, _> = static_copy_map! {
    ///     false => 0,
    ///     true => 1,
    /// };
    /// let map: &StaticMap<_, _> = map.as_copy();
    /// assert_eq!(map[false], 0);
    /// assert_eq!(map[true], 1);
    /// ```
    #[inline]
    pub fn as_static_map(&self) -> &StaticMap<L, T>
    where
        T: Copy,
    {
        StaticMap::from_ref(self.0.as_storage())
    }

    /// Converts a mutable reference to this map to a mutable reference to a [StaticMap].
    ///
    /// This is a zero-cost re-interpretation conversion.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use linearize::{static_copy_map, StaticCopyMap, StaticMap};
    /// let mut map: StaticCopyMap<_, _> = static_copy_map! {
    ///     false => 0,
    ///     true => 1,
    /// };
    /// assert_eq!(map[false], 0);
    /// assert_eq!(map[true], 1);
    /// {
    ///     let map: &mut StaticMap<_, _> = map.as_static_map_mut();
    ///     map[false] = 1;
    ///     map[true] = 0;
    /// }
    /// assert_eq!(map[false], 1);
    /// assert_eq!(map[true], 0);
    /// ```
    #[inline]
    pub fn as_static_map_mut(&mut self) -> &mut StaticMap<L, T>
    where
        T: Copy,
    {
        StaticMap::from_mut(self.0.as_storage_mut())
    }

    /// Remaps the values of this type.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use linearize::{static_copy_map, StaticCopyMap, StaticMap};
    /// let map: StaticCopyMap<_, u8> = static_copy_map! {
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
    pub fn map<U>(self, map: impl FnMut(L, T) -> U) -> StaticCopyMap<L, U>
    where
        L: Sized,
        U: Copy,
    {
        self.into_static_map().map(map).into_copy()
    }

    /// Remaps the values of this type without retrieving the keys.
    ///
    /// If you don't need access to the keys, this function can be more efficient.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use linearize::{static_copy_map, StaticCopyMap, StaticMap};
    /// let map: StaticCopyMap<_, u8> = static_copy_map! {
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
    pub fn map_values<U>(self, map: impl FnMut(T) -> U) -> StaticCopyMap<L, U>
    where
        U: Copy,
    {
        self.into_static_map().map_values(map).into_copy()
    }

    /// Consumes the map and returns an iterator over the values.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use linearize::{static_copy_map, StaticCopyMap};
    /// let mut map: StaticCopyMap<_, u8> = static_copy_map! {
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
        self.into_static_map().into_values()
    }
}

impl<L, T> Deref for StaticCopyMap<L, T>
where
    L: Linearize + ?Sized,
    T: Copy,
{
    type Target = StaticMap<L, T>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        StaticMap::from_ref(self.0.as_storage())
    }
}

impl<L, T> DerefMut for StaticCopyMap<L, T>
where
    L: Linearize + ?Sized,
    T: Copy,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        StaticMap::from_mut(self.0.as_storage_mut())
    }
}

impl<L, T> Clone for StaticCopyMap<L, T>
where
    L: Linearize + ?Sized,
    T: Copy,
{
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<L, T> Index<&'_ L> for StaticCopyMap<L, T>
where
    L: Linearize + ?Sized,
    T: Copy,
{
    type Output = T;

    #[inline]
    fn index(&self, index: &L) -> &Self::Output {
        self.deref().index(index)
    }
}

impl<L, T> Index<L> for StaticCopyMap<L, T>
where
    L: Linearize,
    T: Copy,
{
    type Output = T;

    #[inline]
    fn index(&self, index: L) -> &Self::Output {
        self.deref().index(index)
    }
}

impl<L, T> Index<Linearized<L>> for StaticCopyMap<L, T>
where
    L: Linearize + ?Sized,
    T: Copy,
{
    type Output = T;

    fn index(&self, index: Linearized<L>) -> &Self::Output {
        self.deref().index(index)
    }
}

impl<L, T> IndexMut<&'_ L> for StaticCopyMap<L, T>
where
    L: Linearize + ?Sized,
    T: Copy,
{
    #[inline]
    fn index_mut(&mut self, index: &L) -> &mut Self::Output {
        self.deref_mut().index_mut(index)
    }
}

impl<L, T> IndexMut<L> for StaticCopyMap<L, T>
where
    L: Linearize,
    T: Copy,
{
    #[inline]
    fn index_mut(&mut self, index: L) -> &mut Self::Output {
        self.deref_mut().index_mut(index)
    }
}

impl<L, T> IndexMut<Linearized<L>> for StaticCopyMap<L, T>
where
    L: Linearize + ?Sized,
    T: Copy,
{
    fn index_mut(&mut self, index: Linearized<L>) -> &mut Self::Output {
        self.deref_mut().index_mut(index)
    }
}

impl<L, T> Copy for StaticCopyMap<L, T>
where
    L: Linearize + ?Sized,
    T: Copy,
{
}

impl<L, T> FromIterator<(L, T)> for StaticCopyMap<L, T>
where
    L: Linearize,
    T: Default,
    T: Copy,
{
    #[inline]
    fn from_iter<I: IntoIterator<Item = (L, T)>>(iter: I) -> Self {
        StaticMap::<L, T>::from_iter(iter).into_copy()
    }
}

impl<'a, L, T> FromIterator<(&'a L, T)> for StaticCopyMap<L, T>
where
    L: Linearize,
    T: Default,
    T: Copy,
{
    #[inline]
    fn from_iter<I: IntoIterator<Item = (&'a L, T)>>(iter: I) -> Self {
        let mut res = StaticCopyMap::<L, Option<T>>::default();
        for (k, v) in iter {
            res[k] = Some(v);
        }
        res.map_values(|v| v.unwrap_or_default())
    }
}

impl<L, T> AsMut<[T]> for StaticCopyMap<L, T>
where
    L: Linearize + ?Sized,
    T: Copy,
{
    #[inline]
    fn as_mut(&mut self) -> &mut [T] {
        self.deref_mut().as_mut()
    }
}

impl<L, T> AsRef<[T]> for StaticCopyMap<L, T>
where
    L: Linearize + ?Sized,
    T: Copy,
{
    #[inline]
    fn as_ref(&self) -> &[T] {
        self.deref().as_ref()
    }
}

impl<L, T> Borrow<[T]> for StaticCopyMap<L, T>
where
    L: Linearize + ?Sized,
    T: Copy,
{
    #[inline]
    fn borrow(&self) -> &[T] {
        self.deref().borrow()
    }
}

impl<L, T> BorrowMut<[T]> for StaticCopyMap<L, T>
where
    L: Linearize + ?Sized,
    T: Copy,
{
    #[inline]
    fn borrow_mut(&mut self) -> &mut [T] {
        self.deref_mut().borrow_mut()
    }
}

impl<L, T> Debug for StaticCopyMap<L, T>
where
    L: Linearize,
    L: Debug,
    T: Debug,
    T: Copy,
{
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        self.deref().fmt(f)
    }
}

impl<L, T> Default for StaticCopyMap<L, T>
where
    L: Linearize + ?Sized,
    T: Default,
    T: Copy,
{
    #[inline]
    fn default() -> Self {
        StaticMap::<L, T>::default().into_copy()
    }
}

impl<L, T> Eq for StaticCopyMap<L, T>
where
    L: Linearize + ?Sized,
    T: Eq,
    T: Copy,
{
}

impl<L, T> Hash for StaticCopyMap<L, T>
where
    L: Linearize + ?Sized,
    T: Hash,
    T: Copy,
{
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.deref().hash(state)
    }
}

impl<L, T> Ord for StaticCopyMap<L, T>
where
    L: Linearize + ?Sized,
    T: Ord,
    T: Copy,
{
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.deref().cmp(other)
    }

    #[inline]
    fn max(self, other: Self) -> Self
    where
        Self: Sized,
    {
        self.into_static_map()
            .max(other.into_static_map())
            .into_copy()
    }

    #[inline]
    fn min(self, other: Self) -> Self
    where
        Self: Sized,
    {
        self.into_static_map()
            .min(other.into_static_map())
            .into_copy()
    }

    #[inline]
    fn clamp(self, min: Self, max: Self) -> Self
    where
        Self: Sized,
        Self: PartialOrd,
    {
        self.into_static_map()
            .clamp(min.into_static_map(), max.into_static_map())
            .into_copy()
    }
}

impl<L, T> PartialEq for StaticCopyMap<L, T>
where
    L: Linearize + ?Sized,
    T: PartialEq,
    T: Copy,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.deref().eq(other)
    }
}

impl<L, T> PartialOrd for StaticCopyMap<L, T>
where
    L: Linearize + ?Sized,
    T: PartialOrd,
    T: Copy,
{
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.deref().partial_cmp(other)
    }
}

impl<'a, L, T> TryFrom<&'a [T]> for &'a StaticCopyMap<L, T>
where
    L: Linearize + ?Sized,
    T: Copy,
{
    type Error = TryFromSliceError;

    #[inline]
    fn try_from(value: &'a [T]) -> Result<Self, Self::Error> {
        <&StaticMap<L, T>>::try_from(value).map(|v| v.as_copy())
    }
}

impl<'a, L, T> TryFrom<&'a mut [T]> for &'a mut StaticCopyMap<L, T>
where
    L: Linearize + ?Sized,
    T: Copy,
{
    type Error = TryFromSliceError;

    #[inline]
    fn try_from(value: &'a mut [T]) -> Result<Self, Self::Error> {
        <&mut StaticMap<L, T>>::try_from(value).map(|v| v.as_copy_mut())
    }
}

impl<L, T> TryFrom<&[T]> for StaticCopyMap<L, T>
where
    L: Linearize + ?Sized,
    T: Copy,
{
    type Error = TryFromSliceError;

    #[inline]
    fn try_from(value: &[T]) -> Result<Self, Self::Error> {
        StaticMap::try_from(value).map(|v| v.into_copy())
    }
}

impl<L, T> TryFrom<&mut [T]> for StaticCopyMap<L, T>
where
    L: Linearize + ?Sized,
    T: Copy,
{
    type Error = TryFromSliceError;

    #[inline]
    fn try_from(value: &mut [T]) -> Result<Self, Self::Error> {
        StaticMap::try_from(value).map(|v| v.into_copy())
    }
}

#[cfg(feature = "std")]
impl<L, T> TryFrom<Vec<T>> for StaticCopyMap<L, T>
where
    L: Linearize + ?Sized,
    T: Copy,
{
    type Error = Vec<T>;

    #[inline]
    fn try_from(value: Vec<T>) -> Result<Self, Self::Error> {
        StaticMap::try_from(value).map(|v| v.into_copy())
    }
}

impl<L, T> Extend<(L, T)> for StaticCopyMap<L, T>
where
    L: Linearize,
    T: Copy,
{
    #[inline]
    fn extend<I: IntoIterator<Item = (L, T)>>(&mut self, iter: I) {
        self.deref_mut().extend(iter);
    }
}

impl<'a, L, T> Extend<(&'a L, &'a T)> for StaticCopyMap<L, T>
where
    L: Linearize + ?Sized,
    T: Clone,
    T: Copy,
{
    #[inline]
    fn extend<I: IntoIterator<Item = (&'a L, &'a T)>>(&mut self, iter: I) {
        self.deref_mut().extend(iter);
    }
}

impl<'a, L, T> IntoIterator for &'a StaticCopyMap<L, T>
where
    L: Linearize,
    T: 'a,
    T: Copy,
{
    type Item = (L, &'a T);
    type IntoIter = Iter<'a, L, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.deref().into_iter()
    }
}

impl<'a, L, T> IntoIterator for &'a mut StaticCopyMap<L, T>
where
    L: Linearize,
    T: Copy,
{
    type Item = (L, &'a mut T);
    type IntoIter = IterMut<'a, L, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.deref_mut().into_iter()
    }
}

impl<L, T> IntoIterator for StaticCopyMap<L, T>
where
    L: Linearize,
    T: Copy,
{
    type Item = (L, T);
    type IntoIter = IntoIter<L, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.into_static_map().into_iter()
    }
}

impl<L, T> From<StaticMap<L, T>> for StaticCopyMap<L, T>
where
    L: Linearize,
    T: Copy,
{
    #[inline]
    fn from(value: StaticMap<L, T>) -> Self {
        value.into_copy()
    }
}

impl<L, T> AsRef<StaticMap<L, T>> for StaticCopyMap<L, T>
where
    L: Linearize,
    T: Copy,
{
    fn as_ref(&self) -> &StaticMap<L, T> {
        self.as_static_map()
    }
}

impl<L, T> AsMut<StaticMap<L, T>> for StaticCopyMap<L, T>
where
    L: Linearize,
    T: Copy,
{
    fn as_mut(&mut self) -> &mut StaticMap<L, T> {
        self.as_static_map_mut()
    }
}

impl<L, T> Borrow<StaticMap<L, T>> for StaticCopyMap<L, T>
where
    L: Linearize,
    T: Copy,
{
    fn borrow(&self) -> &StaticMap<L, T> {
        self.as_static_map()
    }
}

impl<L, T> BorrowMut<StaticMap<L, T>> for StaticCopyMap<L, T>
where
    L: Linearize,
    T: Copy,
{
    fn borrow_mut(&mut self) -> &mut StaticMap<L, T> {
        self.as_static_map_mut()
    }
}
