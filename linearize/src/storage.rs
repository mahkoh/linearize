use {
    crate::Linearize,
    core::{
        array::TryFromSliceError,
        borrow::{Borrow, BorrowMut},
        cmp::Ordering,
        hash::Hash,
        mem::MaybeUninit,
        ptr,
    },
    std::mem::{transmute_copy, ManuallyDrop},
};

pub trait Storage<L, T>:
    Sized
    + AsRef<[T]>
    + AsMut<[T]>
    + Borrow<[T]>
    + BorrowMut<[T]>
    + IntoIterator<Item = T, IntoIter: ExactSizeIterator + DoubleEndedIterator>
where
    L: Linearize<Storage<T> = Self> + ?Sized,
{
    fn into_copy(self) -> L::CopyStorage<T>
    where
        T: Copy;

    fn as_copy(&self) -> &L::CopyStorage<T>
    where
        T: Copy;

    fn as_copy_mut(&mut self) -> &mut L::CopyStorage<T>
    where
        T: Copy;

    fn into_storage(self) -> L::Storage<T>;

    fn from_fn(cb: impl FnMut(usize) -> T) -> Self;

    fn each_ref(&self) -> <L as Linearize>::Storage<&T>;

    fn each_mut(&mut self) -> <L as Linearize>::Storage<&mut T>;

    fn map<U>(self, cb: impl FnMut(usize, T) -> U) -> <L as Linearize>::Storage<U>;

    fn zip<U>(self, rhs: L::Storage<U>) -> L::Storage<(T, U)>;

    fn unzip<U>(zipped: L::Storage<(T, U)>) -> (Self, L::Storage<U>);

    fn clone(&self) -> Self
    where
        T: Clone;

    fn clone_from(&mut self, source: &Self)
    where
        T: Clone;

    fn default() -> Self
    where
        T: Default;

    fn eq(&self, other: &Self) -> bool
    where
        T: PartialEq;

    fn cmp(&self, other: &Self) -> Ordering
    where
        T: Ord;

    fn max(self, other: Self) -> Self
    where
        T: Ord;

    fn min(self, other: Self) -> Self
    where
        T: Ord;

    fn clamp(self, min: Self, max: Self) -> Self
    where
        T: Ord;

    fn partial_cmp(&self, other: &Self) -> Option<Ordering>
    where
        T: PartialOrd;

    fn as_hash(&self) -> impl Hash
    where
        T: Hash;

    fn ref_try_from(from: &[T]) -> Result<&Self, TryFromSliceError>;

    fn mut_try_from(from: &mut [T]) -> Result<&mut Self, TryFromSliceError>;

    fn copy_ref_try_from(from: &[T]) -> Result<Self, TryFromSliceError>
    where
        T: Copy;

    fn copy_mut_try_from(from: &mut [T]) -> Result<Self, TryFromSliceError>
    where
        T: Copy;

    #[cfg(feature = "std")]
    fn vec_try_from(from: Vec<T>) -> Result<Self, Vec<T>>;
}

pub trait CopyStorage<L, T>: Copy
where
    L: Linearize<CopyStorage<T> = Self> + ?Sized,
    T: Copy,
{
    fn into_storage(self) -> L::Storage<T>;

    fn as_storage(&self) -> &L::Storage<T>;

    fn as_storage_mut(&mut self) -> &mut L::Storage<T>;
}

fn storage_from_array<T, U, L, const N: usize>(array: [U; N]) -> L::Storage<U>
where
    L: Linearize<Storage<T> = [T; N]> + ?Sized,
{
    // SAFETY:
    // - if L::Storage<T> = [T;N], then by definition L::Storage<U> = [U;N]
    // - thanks to ManuallyDrop, only one valid copy will ever exist
    unsafe { transmute_copy(&ManuallyDrop::new(array)) }
}

impl<L, T, const N: usize> Storage<L, T> for [T; N]
where
    L: Linearize<Storage<T> = Self> + ?Sized,
{
    fn into_copy(self) -> L::CopyStorage<T>
    where
        T: Copy,
    {
        // Unnecessary because T: Copy and therefore [T; N] has no drop impl.
        let slf = MaybeUninit::new(self);
        unsafe {
            // SAFETY:
            // - L::Storage<T> = Self by the bounds of this impl.
            // - L::Storage<T> = L::CopyStorage<T> by the definition of Linearize.
            ptr::read(slf.as_ptr() as *const L::CopyStorage<T>)
        }
    }

    fn as_copy(&self) -> &L::CopyStorage<T>
    where
        T: Copy,
    {
        unsafe {
            // SAFETY:
            // - L::Storage<T> = Self by the bounds of this impl.
            // - L::Storage<T> = L::CopyStorage<T> by the definition of Linearize.
            &*(self as *const L::Storage<T> as *const L::CopyStorage<T>)
        }
    }

    fn as_copy_mut(&mut self) -> &mut L::CopyStorage<T>
    where
        T: Copy,
    {
        unsafe {
            // SAFETY:
            // - L::Storage<T> = Self by the bounds of this impl.
            // - L::Storage<T> = L::CopyStorage<T> by the definition of Linearize.
            &mut *(self as *mut L::Storage<T> as *mut L::CopyStorage<T>)
        }
    }

    fn into_storage(self) -> L::Storage<T> {
        self
    }

    fn from_fn(cb: impl FnMut(usize) -> T) -> Self {
        core::array::from_fn(cb)
    }

    fn each_ref(&self) -> L::Storage<&T> {
        let res: [&T; N] = <[T; N]>::each_ref(self);
        unsafe {
            // SAFETY:
            // - L::Storage<X> is required to be [X; L::LENGTH].
            // - L::Storage<T> = Self = [T; N] by the where clause of this impl.
            // - With X = T it follows that L::LENGTH = N;
            // - With X = &T it follows that L::Storage<&T> = [&T; L::LENGTH] = [&T; N].
            ptr::read(&res as *const [&T; N] as *const L::Storage<&T>)
        }
    }

    fn each_mut(&mut self) -> L::Storage<&mut T> {
        let res: [&mut T; N] = <[T; N]>::each_mut(self);
        let res: MaybeUninit<[&mut T; N]> = MaybeUninit::new(res);
        unsafe {
            // SAFETY:
            // - L::Storage<X> is required to be [X; L::LENGTH].
            // - L::Storage<T> = [T; N] by the where clause of this impl.
            // - L::Storage<T> = Self = [T; N] by the where clause of this impl.
            // - With X = T it follows that L::LENGTH = N;
            // - With X = &mut T it follows that L::Storage<&mut T> = [&mut T; L::LENGTH] = [&mut T; N].
            ptr::read(res.as_ptr() as *const L::Storage<&mut T>)
        }
    }

    fn map<U>(self, mut cb: impl FnMut(usize, T) -> U) -> L::Storage<U> {
        let mut src = MaybeUninit::<[T; N]>::new(self);
        let src_ptr = src.as_mut_ptr() as *const T;
        let mut res;
        let res_ptr;
        if size_of::<U>() <= size_of::<T>() && align_of::<U>() <= align_of::<T>() {
            // CASE: map-in-place
            res_ptr = src_ptr as *mut U;
        } else {
            // CASE: map-in-new-place
            res = MaybeUninit::<[U; N]>::uninit();
            res_ptr = res.as_mut_ptr() as *mut U;
        }
        for i in 0..N {
            let t = unsafe {
                // SAFETY:
                // - src_ptr points to [T; N] and i < N.
                // - Each array element is read at most once.
                // - In map-in-new-place case, the array is not written to.
                // - In map-in-place case, only U's with index < i have been written so far
                //   and since size_of::<U> <= size_of::<T>, no bytes of the i'th element
                //   have been written to.
                ptr::read(src_ptr.add(i))
            };
            let u = cb(i, t);
            unsafe {
                // SAFETY:
                // - In map-in-new-place case: res_ptr points memory suitable for [U; N].
                // - In map-in-place case: res_ptr points to [T; N] and
                //   size_of::<U> <= size_of::<T> and align_of::<U> <= align_of::<T>.
                //   Therefore, the pointed to object is also suitable to hold a [U; N].
                ptr::write(res_ptr.add(i), u);
            }
        }
        unsafe {
            // SAFETY:
            // - res_ptr points to an object suitable to hold a [U; N]. See the safety
            //   docs for ptr::write above.
            // - This array has been initialized for each i in [0; N).
            // - L::Storage<X> = [X; N] by the where clause of this impl.
            // - With X = T it follows that L::LENGTH = N;
            // - With X = U it follows that L::Storage<U> = [U; L::LENGTH] = [U; N].
            ptr::read(res_ptr as *const L::Storage<U>)
        }
    }

    fn zip<U>(self, rhs: L::Storage<U>) -> L::Storage<(T, U)> {
        let mut left = self.into_iter();
        let mut right = rhs.into_iter();
        let out: [(T, U); N] = core::array::from_fn(|_| {
            // SAFETY:
            // both left and right are core::array::IntoIter<_,N> and will yield exaclty
            // N elements, and the output also contains exatcly N elements
            unsafe {
                (
                    left.next().unwrap_unchecked(),
                    right.next().unwrap_unchecked(),
                )
            }
        });
        // the iterators are always empty so there is nothing to drop,
        // but there have been examples where the optimizer can struggle to realize
        // that this is always true, and this is free
        core::mem::forget((left, right));
        storage_from_array::<T, (T, U), L, N>(out)
    }

    fn unzip<U>(zipped: L::Storage<(T, U)>) -> (Self, L::Storage<U>) {
        // SAFETY:
        // - L::Storage<(T,U)> = [(T,U);N] by the where clause of this impl.
        // - thanks to ManuallyDrop, only one copy will ever exist
        let as_manually_drop: [ManuallyDrop<(T, U)>; N] =
            unsafe { transmute_copy(&ManuallyDrop::new(zipped)) };
        let (out1, out2) = (
            std::array::from_fn(|i| 
                // SAFETY:
                // each item is only read once, and thanks to the ManuallyDrop never 
                // accessed again
                unsafe { (&raw const as_manually_drop[i].0).read() }
            ),
            std::array::from_fn(|i| 
                // SAFETY:
                // each item is only read once, and thanks to the ManuallyDrop never 
                // accessed again
                unsafe { (&raw const as_manually_drop[i].1).read() 
            }),
        );
        (
            storage_from_array::<T, T, L, N>(out1),
            storage_from_array::<T, U, L, N>(out2),
        )
    }

    fn clone(&self) -> Self
    where
        T: Clone,
    {
        <[T; N] as Clone>::clone(self)
    }

    fn clone_from(&mut self, source: &Self)
    where
        T: Clone,
    {
        <[T; N] as Clone>::clone_from(self, source)
    }

    fn default() -> Self
    where
        T: Default,
    {
        core::array::from_fn(|_| T::default())
    }

    fn eq(&self, other: &Self) -> bool
    where
        T: PartialEq,
    {
        <[T; N] as PartialEq>::eq(self, other)
    }

    fn cmp(&self, other: &Self) -> Ordering
    where
        T: Ord,
    {
        <[T; N] as Ord>::cmp(self, other)
    }

    fn max(self, other: Self) -> Self
    where
        T: Ord,
    {
        <[T; N] as Ord>::max(self, other)
    }

    fn min(self, other: Self) -> Self
    where
        T: Ord,
    {
        <[T; N] as Ord>::min(self, other)
    }

    fn clamp(self, min: Self, max: Self) -> Self
    where
        T: Ord,
    {
        <[T; N] as Ord>::clamp(self, min, max)
    }

    fn partial_cmp(&self, other: &Self) -> Option<Ordering>
    where
        T: PartialOrd,
    {
        <[T; N] as PartialOrd>::partial_cmp(self, other)
    }

    fn as_hash(&self) -> impl Hash
    where
        T: Hash,
    {
        self
    }

    fn ref_try_from(from: &[T]) -> Result<&Self, TryFromSliceError> {
        <&[T; N] as TryFrom<&[T]>>::try_from(from)
    }

    fn mut_try_from(from: &mut [T]) -> Result<&mut Self, TryFromSliceError> {
        <&mut [T; N] as TryFrom<&mut [T]>>::try_from(from)
    }

    fn copy_ref_try_from(from: &[T]) -> Result<Self, TryFromSliceError>
    where
        T: Copy,
    {
        <[T; N] as TryFrom<&[T]>>::try_from(from)
    }

    fn copy_mut_try_from(from: &mut [T]) -> Result<Self, TryFromSliceError>
    where
        T: Copy,
    {
        <[T; N] as TryFrom<&mut [T]>>::try_from(from)
    }

    #[cfg(feature = "std")]
    fn vec_try_from(from: Vec<T>) -> Result<Self, Vec<T>> {
        <[T; N] as TryFrom<Vec<T>>>::try_from(from)
    }
}

impl<L, T, const N: usize> CopyStorage<L, T> for [T; N]
where
    L: Linearize<Storage<T> = Self, CopyStorage<T> = Self> + ?Sized,
    T: Copy,
{
    fn into_storage(self) -> L::Storage<T> {
        self
    }

    fn as_storage(&self) -> &L::Storage<T> {
        self
    }

    fn as_storage_mut(&mut self) -> &mut L::Storage<T> {
        self
    }
}
