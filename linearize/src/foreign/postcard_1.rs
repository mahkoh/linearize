//! Implement the [`MaxSize`] trait from [`postcard_1`] for [`StaticMap`] and [`StaticCopyMap`].
//!
//! The table below shows how the serialization impls correspond to postcard's serializer and max size calculation.
//! | serde impl                | postcard serializer        | max bytes                        |
//! |---------------------------|----------------------------|----------------------------------|
//! | serialize_map(len)        | try_push_varint_usize(len) | usize::POSTCARD_MAX_SIZE         |
//! | serialize_key(&k) * len   | k.serialize()              | L::POSTCARD_MAX_SIZE * L::LENGTH |
//! | serialize_value(&v) * len | v.serialize()              | T::POSTCARD_MAX_SIZE * L::LENGTH |

use {
    crate::{Linearize, StaticCopyMap, StaticMap},
    postcard_1::experimental::max_size::MaxSize,
};

impl<L, T> MaxSize for StaticMap<L, T>
where
    L: Linearize + MaxSize,
    T: MaxSize,
{
    const POSTCARD_MAX_SIZE: usize =
        usize::POSTCARD_MAX_SIZE + L::LENGTH * (L::POSTCARD_MAX_SIZE + T::POSTCARD_MAX_SIZE);
}

impl<L, T> MaxSize for StaticCopyMap<L, T>
where
    L: Linearize + MaxSize,
    T: MaxSize + Copy,
{
    const POSTCARD_MAX_SIZE: usize = <StaticMap<L, T>>::POSTCARD_MAX_SIZE;
}
