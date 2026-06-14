use {
    crate::{Linearize, StaticCopyMap, StaticMap},
    core::ops::Deref,
    defmt_1 as defmt,
};

impl<L, T> defmt::Format for StaticMap<L, T>
where
    L: Linearize + defmt::Format,
    T: defmt::Format,
{
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "{{");
        let mut first = true;
        for (k, v) in self {
            if !first {
                defmt::write!(fmt, ", ");
            }
            first = false;
            defmt::write!(fmt, "{}: {}", k, v);
        }
        defmt::write!(fmt, "}}");
    }
}

impl<L, T> defmt::Format for StaticCopyMap<L, T>
where
    L: Linearize + defmt::Format,
    T: defmt::Format + Copy,
{
    fn format(&self, fmt: defmt::Formatter) {
        self.deref().format(fmt);
    }
}
