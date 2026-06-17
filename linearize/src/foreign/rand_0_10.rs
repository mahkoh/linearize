#[cfg(feature = "alloc")]
use rand_0_10::distr::weighted::WeightedIndex;
use {
    crate::{static_copy_map, static_map, Linearize, StaticCopyMap, StaticMap},
    rand_0_10::{
        distr::{
            uniform::{SampleUniform, Uniform},
            Bernoulli, Open01, OpenClosed01, StandardUniform,
        },
        prelude::Distribution,
        Rng,
    },
};

macro_rules! impl_distribution {
    ($dist:ident, $map:ident, $constructor:ident, $($bounds:tt)*) => {
        impl<L, T> Distribution<$map<L, T>> for $dist
        where
            $dist: Distribution<T>,
            L: Linearize,
            T: $($bounds)*,
        {
            fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> $map<L, T> {
                $constructor! {
                    _ => self.sample(rng),
                }
            }
        }
    };
}

macro_rules! impl_distributions {
    ($map:ident, $constructor:ident, $($bounds:tt)*) => {
        impl_distribution!(StandardUniform, $map, $constructor, $($bounds)*);
        impl_distribution!(Open01, $map, $constructor, $($bounds)*);
        impl_distribution!(OpenClosed01, $map, $constructor, $($bounds)*);
        impl_distribution!(Bernoulli, $map, $constructor, $($bounds)*);

        impl<L, T, X> Distribution<$map<L, T>> for Uniform<X>
        where
            X: SampleUniform,
            Uniform<X>: Distribution<T>,
            L: Linearize,
            T: $($bounds)*,
        {
            fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> $map<L, T> {
                $constructor! {
                    _ => self.sample(rng),
                }
            }
        }

        #[cfg(feature = "alloc")]
        impl<L, T, X> Distribution<$map<L, T>> for WeightedIndex<X>
        where
            X: SampleUniform + PartialOrd,
            WeightedIndex<X>: Distribution<T>,
            L: Linearize,
            T: $($bounds)*,
        {
            fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> $map<L, T> {
                $constructor! {
                    _ => self.sample(rng),
                }
            }
        }
    }
}

impl_distributions!(StaticCopyMap, static_copy_map, Copy);
impl_distributions!(StaticMap, static_map,);
