macro_rules! assert_auto_trait {
    ($bound:ident) => {
        const _: () = {
            use linearize::{Linearize, StaticCopyMap, StaticMap};

            fn _assert<T: $bound>() {}

            fn _forward<L, T>()
            where
                L: Linearize + ?Sized,
                L::Storage<T>: $bound,
            {
                _assert::<StaticMap<L, T>>();
            }

            fn _forward_copy<L, T>()
            where
                L: Linearize + ?Sized,
                L::CopyStorage<T>: $bound,
                T: Copy,
            {
                _assert::<StaticCopyMap<L, T>>();
            }
        };
    };
}

assert_auto_trait!(Send);
assert_auto_trait!(Sync);
