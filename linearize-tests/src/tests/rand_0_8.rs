use {
    linearize::{static_map, StaticCopyMap, StaticMap},
    rand_0_8::{
        distributions::{
            uniform::SampleUniform, Bernoulli, Distribution, Open01, OpenClosed01, Standard,
            Uniform,
        },
        random,
    },
};

macro_rules! assert_forwards {
    ($ty:ty $(, $generic:ident: $bound:ident)?) => {
        const _: () = {
            #[allow(unconditional_recursion)]
            fn _forwards<T, $($generic: $bound)?>()
            where
                $ty: Distribution<T>,
            {
                _forwards::<StaticMap<(), T>, $($generic)?>();
            }

            #[allow(unconditional_recursion)]
            fn _forwards_copy<T, $($generic: $bound)?>()
            where
                T: Copy,
                $ty: Distribution<T>,
            {
                _forwards_copy::<StaticCopyMap<(), T>, $($generic)?>();
            }
        };
    };
}

assert_forwards!(Standard);
assert_forwards!(Open01);
assert_forwards!(OpenClosed01);
assert_forwards!(Bernoulli);
assert_forwards!(Uniform<X>, X: SampleUniform);

#[test]
fn smoketest() {
    let v = static_map! {
        true => random::<u8>(),
        _ => random(),
    };
    dbg!(v);
}
