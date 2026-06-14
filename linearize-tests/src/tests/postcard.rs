use {
    linearize::{static_map, StaticCopyMap, StaticMap},
    postcard::experimental::max_size::MaxSize,
};

#[test]
fn max_size_is_upper_bound() {
    // Small map: bool keys (2 entries)
    let map: StaticMap<bool, u8> = static_map! {
        false => u8::MAX,
        true => u8::MAX,
    };
    let mut buf = [0u8; <StaticMap<bool, u8>>::POSTCARD_MAX_SIZE];
    postcard::to_slice(&map, &mut buf).unwrap();

    // Medium map with larger values
    let map: StaticMap<bool, u16> = static_map! {
        false => u16::MAX,
        true => u16::MAX,
    };
    let mut buf = [0u8; <StaticMap<bool, u16>>::POSTCARD_MAX_SIZE];
    postcard::to_slice(&map, &mut buf).unwrap();

    // Large map: u8 keys (256 entries), max values
    let map = StaticMap::<u8, u8>::from_fn(|_| u8::MAX);
    let mut buf = [0u8; <StaticMap<u8, u8>>::POSTCARD_MAX_SIZE];
    postcard::to_slice(&map, &mut buf).unwrap();

    // CopyMap variant
    let map: StaticCopyMap<bool, u8> = StaticCopyMap::from_fn(|_| u8::MAX);
    let mut buf = [0u8; <StaticCopyMap<bool, u8>>::POSTCARD_MAX_SIZE];
    postcard::to_slice(&map, &mut buf).unwrap();
}
