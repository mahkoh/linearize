use {
    defmt::Format,
    linearize::{StaticCopyMap, StaticMap},
};

const _: () = {
    #[allow(unconditional_recursion)]
    fn _forward<T: Format>() {
        _forward::<StaticMap<(), T>>();
    }

    #[allow(unconditional_recursion)]
    fn _forward_copy<T: Format + Copy>() {
        _forward_copy::<StaticCopyMap<(), T>>();
    }
};
