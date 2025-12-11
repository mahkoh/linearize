# linearize

[![crates.io](https://img.shields.io/crates/v/linearize.svg)](http://crates.io/crates/linearize)
[![docs.rs](https://docs.rs/linearize/badge.svg)](http://docs.rs/linearize)
![MSRV](https://img.shields.io/crates/msrv/linearize)

This crate provides a trait that defines an enumeration of a type and an efficient no_std
map that uses such types as keys.

## Example

```rust
use linearize::{Linearize, static_map};

#[derive(Linearize)]
enum Keys {
    A,
    B(bool),
}

fn main() {
    let map = static_map! {
        Keys::A => "a",
        Keys::B(false) => "b",
        Keys::B(true) => "c",
    };
    assert_eq!(map[Keys::A], "a");
    assert_eq!(map[Keys::B(true)], "c");
}
```

## MSRV

The MSRV is `stable - 3`.

## License

This project is licensed under either of

- Apache License, Version 2.0
- MIT License

at your option.
