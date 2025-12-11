use {
    bytemuck::{AnyBitPattern, NoUninit, Zeroable},
    linearize::StaticCopyMap,
    std::marker::PhantomData,
};

macro_rules! assert_forwards {
    ($($trait:tt)*) => {
        const _: () = {
            #[allow(unconditional_recursion)]
            fn _forward<T: $($trait)*>() {
                _forward::<StaticCopyMap<(), T>>();
            }
        };
    };
}

assert_forwards!(AnyBitPattern);
assert_forwards!(NoUninit);
assert_forwards!(Zeroable + Copy);

macro_rules! assert_not_forwards {
    (($($trait:tt)*), ($($others:tt)*)) => {
        #[expect(dead_code)]
        const _: () = {
            trait T1 {
                const A: usize;
            }

            trait T2 {
                const A: usize;
            }

            struct S<T>(PhantomData<T>);

            impl<T> T1 for S<T> {
                const A: usize = 0;
            }

            impl<T: $($trait)*> T2 for S<T> {
                const A: usize = 0;
            }

            fn _no_forward<T: $($others)*>() {
                let _ = S::<StaticCopyMap<(), T>>::A;
            }
        };
    };
}

assert_not_forwards!((AnyBitPattern), (NoUninit + Zeroable + Copy));
assert_not_forwards!((NoUninit), (AnyBitPattern + Zeroable + Copy));
assert_not_forwards!((Zeroable), (NoUninit + Copy));
