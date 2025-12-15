#[macro_use]
mod utils;
mod arbitrary;
mod bytemuck;
mod copy_map;
mod core_addressing;
mod derive;
mod linearize_ext;
mod linearized;
mod r#macro;
mod map;
mod rand;
mod serde;
mod variants;

mod test {
    #[expect(dead_code)]
    struct S {
        a: bool,
        b: u8,
    }

    #[allow(clippy::modulo_one, clippy::manual_range_contains)]
    const _: () = {
        trait __C {
            const B0: usize;
            const B1: usize;
        }
        impl __C for S
        where
            bool: ::linearize::Linearize,
            u8: ::linearize::Linearize,
        {
            const B0: usize = 0;
            const B1: usize = 1usize
                * <u8 as ::linearize::Linearize>::LENGTH
                * <bool as ::linearize::Linearize>::LENGTH;
        }
        #[automatically_derived]
        unsafe impl ::linearize::Linearize for S
        where
            bool: ::linearize::Linearize,
            u8: ::linearize::Linearize,
        {
            type Storage<__T> = [__T; <Self as ::linearize::Linearize>::LENGTH];
            type CopyStorage<__T>
                = [__T; <Self as ::linearize::Linearize>::LENGTH]
            where
                __T: Copy;
            const LENGTH: usize = <Self as __C>::B1;
            #[inline]
            fn linearize(&self) -> usize {
                let mut res = <Self as __C>::B0;
                res = res.wrapping_add(
                    <u8 as ::linearize::Linearize>::linearize(&self.b)
                        .wrapping_mul(const { 1usize }),
                );
                res = res.wrapping_add(
                    <bool as ::linearize::Linearize>::linearize(&self.a)
                        .wrapping_mul(const { 1usize * <u8 as ::linearize::Linearize>::LENGTH }),
                );
                res
            }
            #[inline]
            unsafe fn from_linear_unchecked(linear: usize) -> Self {
                Self {
                    a: {
                        let idx = (linear
                            / const { 1usize * <u8 as ::linearize::Linearize>::LENGTH })
                            % <bool as ::linearize::Linearize>::LENGTH;
                        <bool as ::linearize::Linearize>::from_linear_unchecked(idx)
                    },
                    b: {
                        let idx =
                            (linear / const { 1usize }) % <u8 as ::linearize::Linearize>::LENGTH;
                        <u8 as ::linearize::Linearize>::from_linear_unchecked(idx)
                    },
                }
            }
        }
    };
}
