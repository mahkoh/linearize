#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(clippy::missing_safety_doc)]

//! A crate for enumerable types.
//!
//! This crate provides the [Linearize] trait which defines a bijection between a type
//! and an interval of the natural numbers.
//!
//! Given such a bijection, many useful things become possible. For example, this crate
//! defines the types [StaticMap] and [StaticCopyMap] which provide high-performance,
//! non-allocating mappings from linearizable types to arbitrary values.
//!
//! ```rust
//! # use linearize::StaticMap;
//! # use linearize_derive::Linearize;
//! #
//! #[derive(Linearize)]
//! enum ColorFormat {
//!     R,
//!     Rgb {
//!         alpha: bool,
//!     },
//! }
//!
//! let mut channels = StaticMap::default();
//! channels[ColorFormat::R] = 1;
//! channels[ColorFormat::Rgb { alpha: false }] = 3;
//! channels[ColorFormat::Rgb { alpha: true }] = 4;
//!
//! assert_eq!(channels[ColorFormat::Rgb { alpha: false }], 3);
//! ```
//!
//! These maps can be constructed conveniently with the [static_map] macro:
//!
//! ```rust
//! # use linearize::{static_map, StaticMap};
//! # use linearize_derive::Linearize;
//! #
//! # #[derive(Linearize)]
//! # enum ColorFormat {
//! #     R,
//! #     Rgb {
//! #         alpha: bool,
//! #     },
//! # }
//! #
//! let channels = static_map! {
//!     ColorFormat::R => 1,
//!     ColorFormat::Rgb { alpha } => 3 + alpha as u32,
//! };
//!
//! assert_eq!(channels[ColorFormat::Rgb { alpha: false }], 3);
//! ```
//!
//! # Features
//!
//! The following features are enabled by default:
//!
//! - `std`
//!
//! This crate provides the following features:
//!
//! - `alloc`: Adds a dependency on the `alloc` crate. This implements additional traits
//!   for the map types.
//! - `std`: Adds a dependency on the `std` crate.
//! - `derive`: Provides the [Linearize](linearize_derive::Linearize) derive macro.
//! - `serde-1`: Implements `Serialize` and `Deserialize` from serde 1.x for the map types.
//! - `arbitrary-1`: Implements `Arbitrary` from arbitrary 1.x for the map types.
//! - `bytemuck-1`: Implements `NoUninit`, `Zeroable`, and `AnyBitPattern` from bytemuck 1.x for the map types.
//! - `rand-0_8`: Implements various distributions from rand 0.8.x for the map types.
//! - `rand-0_9`: Implements various distributions from rand 0.9.x for the map types.

#[cfg(feature = "alloc")]
extern crate alloc;

mod copy_map;
mod foreign;
mod impls;
mod linearized;
mod r#macro;
mod map;
mod storage;
mod variants;

use crate::{
    sealed::Sealed,
    storage::{CopyStorage, Storage},
    variants::Variants,
};
#[cfg(feature = "serde-1")]
pub use foreign::serde_1;
#[cfg(feature = "derive")]
pub use linearize_derive::Linearize;
#[doc(hidden)]
pub use r#macro::Builder;
pub use {copy_map::StaticCopyMap, linearized::Linearized, map::StaticMap};

/// Types whose values can be enumerated.
///
/// Types that implement this trait define a bijection between themselves and an interval
/// of the natural numbers.
///
/// # Safety
///
/// - [`Self::Storage<T>`] must be `[T; Self::LENGTH]`.
/// - [`Self::CopyStorage<T>`] must be `[T; Self::LENGTH]`.
/// - [`Self::linearize`] must be a bijection to `[0, Self::LENGTH)`.
/// - [`Self::from_linear_unchecked`] must be its inverse.
///
/// Note that the bijection implies that a roundtrip through
/// `linearize | from_linear_unchecked` must return a value that is, for all intents and
/// purposes, indistinguishable from the original value. The details of this depend on
/// `Self`.
pub unsafe trait Linearize {
    /// `[T; Self::LENGTH]`
    ///
    /// This type exists due to a limitation of the rust type system. In a future version
    /// of this crate, all uses of it will be replaced by `[T; Self::LENGTH]`.
    type Storage<T>: Storage<Self, T>;

    /// `[T; Self::LENGTH]`
    ///
    /// This type exists due to a limitation of the rust type system. In a future version
    /// of this crate, all uses of it will be replaced by `[T; Self::LENGTH]`.
    type CopyStorage<T>: CopyStorage<Self, T>
    where
        T: Copy;

    /// The cardinality of this type.
    const LENGTH: usize;

    /// Maps this value to the natural numbers.
    ///
    /// This function is a bijection to the interval `[0, Self::LENGTH)`.
    fn linearize(&self) -> usize;

    /// The inverse of the `linearize` function.
    ///
    /// # Safety
    ///
    /// `linear` must be less than [`Self::LENGTH`].
    unsafe fn from_linear_unchecked(linear: usize) -> Self
    where
        Self: Sized;
}

/// Extension trait for types implementing [Linearize].
pub trait LinearizeExt: Linearize + Sealed {
    /// A safe version of [Linearize::from_linear_unchecked].
    ///
    /// This function returns `None` if `linear >= Self::LENGTH`.
    fn from_linear(linear: usize) -> Option<Self>
    where
        Self: Sized;

    /// Returns an iterator over all values of this type.
    fn variants() -> Variants<Self>
    where
        Self: Sized;

    /// Linearizes this value and stores the value in a [Linearized] object.
    ///
    /// See the documentation of [Linearized] for why this might be useful.
    fn linearized(&self) -> Linearized<Self>;
}

impl<T> LinearizeExt for T
where
    T: Linearize + ?Sized,
{
    fn from_linear(linear: usize) -> Option<Self>
    where
        Self: Sized,
    {
        (linear < Self::LENGTH).then(|| unsafe {
            // SAFETY: This closure is only called if linear < Self::LENGTH.
            Self::from_linear_unchecked(linear)
        })
    }

    fn variants() -> Variants<Self>
    where
        Self: Sized,
    {
        Variants::new()
    }

    fn linearized(&self) -> Linearized<Self> {
        Linearized::new(self)
    }
}

impl<T> Sealed for T where T: Linearize + ?Sized {}

mod sealed {
    pub trait Sealed {}
}

pub mod iter {
    //! All iterators exposed by this crate.
    //!
    //! This module exists only to keep the top-level namespace clean.
    pub use crate::{
        map::iters::{IntoIter, IntoValues, Iter, IterMut},
        variants::Variants,
    };
}
