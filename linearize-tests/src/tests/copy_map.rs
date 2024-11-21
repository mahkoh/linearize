use {
    linearize::{static_copy_map, static_map, LinearizeExt, StaticCopyMap, StaticMap},
    std::{
        borrow::{Borrow, BorrowMut},
        cmp::Ordering,
        collections::HashMap,
        hash::{BuildHasher, RandomState},
        ops::{Deref, DerefMut},
    },
};

#[test]
fn map() {
    let a = static_copy_map! {
        false => 1,
        true => 2,
    };
    let x = a.map_values(|v| v * 2);
    assert_eq!(x[false], 2);
    assert_eq!(x[true], 4);
    let x = a.map(|b, v| b as i32 + v);
    assert_eq!(x[false], 1);
    assert_eq!(x[true], 3);
}

#[test]
fn from_fn() {
    let map = StaticCopyMap::from_fn(|k: bool| k as usize);
    assert_eq!(map[false], 0);
    assert_eq!(map[true], 1);
}

#[test]
fn from_ref() {
    let array = [0, 1];
    let map = StaticCopyMap::from_ref(&array);
    assert_eq!(map[false], 0);
    assert_eq!(map[true], 1);
}

#[test]
fn from_mut() {
    let mut array = [0, 1];
    let map = StaticCopyMap::from_mut(&mut array);
    map[false] = 1;
    map[true] = 0;
    assert_eq!(array, [1, 0]);
}

#[test]
fn into_static_map() {
    let map: StaticCopyMap<_, _> = static_copy_map! {
        false => 0,
        true => 1,
    };
    let map: StaticMap<_, _> = map.into_static_map();
    assert_eq!(map[false], 0);
    assert_eq!(map[true], 1);
}

#[test]
fn as_static_map() {
    let map: StaticCopyMap<_, _> = static_copy_map! {
        false => 0,
        true => 1,
    };
    let map: &StaticMap<_, _> = map.as_static_map();
    assert_eq!(map[false], 0);
    assert_eq!(map[true], 1);
}

#[test]
fn as_static_map_mut_mut() {
    let mut map: StaticCopyMap<_, _> = static_copy_map! {
        false => 0,
        true => 1,
    };
    assert_eq!(map[false], 0);
    assert_eq!(map[true], 1);
    {
        let map: &mut StaticMap<_, _> = map.as_static_map_mut();
        map[false] = 1;
        map[true] = 0;
    }
    assert_eq!(map[false], 1);
    assert_eq!(map[true], 0);
}

#[test]
fn map_values() {
    let map: StaticCopyMap<_, u8> = static_copy_map! {
        false => 0,
        true => 1,
    };
    assert_eq!(map[false], 0);
    assert_eq!(map[true], 1);
    let map = map.map_values(|v| 3 * v);
    assert_eq!(map[false], 0);
    assert_eq!(map[true], 3);
}

#[test]
fn deref() {
    let mut map1: StaticCopyMap<_, u8> = static_copy_map! {
        false => 0,
        true => 1,
    };
    let mut map2: StaticMap<_, u8> = static_map! {
        false => 0,
        true => 1,
    };
    assert_eq!(map1.deref(), &map2);
    assert_eq!(map1.deref_mut(), &mut map2);
}

#[test]
fn from_iter() {
    let map: StaticCopyMap<_, u8> = [(false, 1)].into_iter().collect();
    assert_eq!(map[false], 1);
    assert_eq!(map[true], 0);
}

#[test]
fn index() {
    let mut map: StaticCopyMap<_, u8> = static_copy_map! {
        false => 0,
        true => 1,
    };
    map[false] = 3;
    map[true] = 4;
    assert_eq!(map[false], 3);
    assert_eq!(map[true], 4);
    map[&false] = 5;
    map[&true] = 6;
    assert_eq!(map[&false], 5);
    assert_eq!(map[&true], 6);
    map[false.linearized()] = 7;
    map[true.linearized()] = 8;
    assert_eq!(map[false.linearized()], 7);
    assert_eq!(map[true.linearized()], 8);
}

#[test]
fn as_ref() {
    let mut map: StaticCopyMap<_, u8> = static_copy_map! {
        false => 0,
        true => 1,
    };
    let mut static_map = map.into_static_map();
    assert_eq!(AsRef::<[u8]>::as_ref(&map), &[0, 1]);
    assert_eq!(AsMut::<[u8]>::as_mut(&mut map), &mut [0, 1]);
    assert_eq!(AsRef::<StaticMap<_, _>>::as_ref(&map), &static_map);
    assert_eq!(AsMut::<StaticMap<_, _>>::as_mut(&mut map), &mut static_map);
}

#[test]
fn borrow() {
    let mut map: StaticCopyMap<_, u8> = static_copy_map! {
        false => 0,
        true => 1,
    };
    let mut static_map = map.into_static_map();
    assert_eq!(Borrow::<[u8]>::borrow(&map), &[0, 1]);
    assert_eq!(BorrowMut::<[u8]>::borrow_mut(&mut map), &mut [0, 1]);
    assert_eq!(Borrow::<StaticMap<_, _>>::borrow(&map), &static_map);
    assert_eq!(
        BorrowMut::<StaticMap<_, _>>::borrow_mut(&mut map),
        &mut static_map
    );
}

#[test]
fn debug() {
    let map1: StaticCopyMap<(), u8> = static_copy_map! {
        () => 0,
    };
    let mut map2 = HashMap::new();
    map2.insert((), 0);
    assert_eq!(format!("{:?}", map1), format!("{:?}", map2));
}

#[test]
fn default() {
    #[derive(Default, PartialEq, Debug, Copy, Clone)]
    struct X;
    let map = StaticCopyMap::<_, X>::default();
    assert_eq!(map[false], X);
    assert_eq!(map[true], X);
}

#[test]
fn hash() {
    let map: StaticCopyMap<_, u8> = static_copy_map! {
        false => 0,
        true => 1,
    };
    let random = RandomState::new();
    assert_eq!(random.hash_one(map), random.hash_one([0u8, 1]));
}

#[test]
fn ord() {
    let map1: StaticCopyMap<_, u8> = static_copy_map! {
        false => 0,
        true => 1,
    };
    let map2: StaticCopyMap<_, u8> = static_copy_map! {
        false => 1,
        true => 1,
    };
    let map3: StaticCopyMap<_, u8> = static_copy_map! {
        false => 2,
        true => 1,
    };
    assert_eq!(map1.cmp(&map2), Ordering::Less);
    assert_eq!(map1.cmp(&map1), Ordering::Equal);
    assert_eq!(map2.cmp(&map1), Ordering::Greater);
    assert_eq!(map1.clone().max(map2.clone()), map2);
    assert_eq!(map2.clone().max(map1.clone()), map2);
    assert_eq!(map1.clone().min(map2.clone()), map1);
    assert_eq!(map2.clone().min(map1.clone()), map1);
    assert_eq!(map1.clone().clamp(map1.clone(), map1.clone()), map1);
    assert_eq!(map1.clone().clamp(map1.clone(), map2.clone()), map1);
    assert_eq!(map1.clone().clamp(map1.clone(), map3.clone()), map1);
    assert_eq!(map1.clone().clamp(map2.clone(), map2.clone()), map2);
    assert_eq!(map1.clone().clamp(map2.clone(), map3.clone()), map2);
    assert_eq!(map1.clone().clamp(map3.clone(), map3.clone()), map3);
    assert_eq!(map2.clone().clamp(map1.clone(), map1.clone()), map1);
    assert_eq!(map2.clone().clamp(map1.clone(), map2.clone()), map2);
    assert_eq!(map2.clone().clamp(map1.clone(), map3.clone()), map2);
    assert_eq!(map2.clone().clamp(map2.clone(), map2.clone()), map2);
    assert_eq!(map2.clone().clamp(map2.clone(), map3.clone()), map2);
    assert_eq!(map2.clone().clamp(map3.clone(), map3.clone()), map3);
    assert_eq!(map3.clone().clamp(map1.clone(), map1.clone()), map1);
    assert_eq!(map3.clone().clamp(map1.clone(), map2.clone()), map2);
    assert_eq!(map3.clone().clamp(map1.clone(), map3.clone()), map3);
    assert_eq!(map3.clone().clamp(map2.clone(), map2.clone()), map2);
    assert_eq!(map3.clone().clamp(map2.clone(), map3.clone()), map3);
    assert_eq!(map3.clone().clamp(map3.clone(), map3.clone()), map3);
}

#[test]
fn partial_eq() {
    let map1: StaticCopyMap<_, u8> = static_copy_map! {
        false => 0,
        true => 1,
    };
    let map2: StaticCopyMap<_, u8> = static_copy_map! {
        false => 1,
        true => 1,
    };
    assert_eq!(map1.eq(&map1), true);
    assert_eq!(map1.eq(&map2), false);
    assert_eq!(map2.eq(&map1), false);
    assert_eq!(map2.eq(&map2), true);
}

#[test]
fn partial_ord() {
    let map1: StaticCopyMap<_, u8> = static_copy_map! {
        false => 0,
        true => 1,
    };
    let map2: StaticCopyMap<_, u8> = static_copy_map! {
        false => 1,
        true => 1,
    };
    assert_eq!(map1.partial_cmp(&map2), Some(Ordering::Less));
    assert_eq!(map1.partial_cmp(&map1), Some(Ordering::Equal));
    assert_eq!(map2.partial_cmp(&map1), Some(Ordering::Greater));

    let map1: StaticCopyMap<_, f64> = static_copy_map! {
        false => f64::NAN,
        true => 1.0,
    };
    let map2: StaticCopyMap<_, f64> = static_copy_map! {
        false => f64::NAN,
        true => 1.0,
    };
    assert_eq!(map1.partial_cmp(&map2), None);
    assert_eq!(map1.partial_cmp(&map1), None);
    assert_eq!(map2.partial_cmp(&map1), None);
}

#[test]
fn try_from() {
    let mut map = static_copy_map! {
        false => 0,
        true => 1,
    };

    let slice: &[u8] = &[0, 1];
    assert_eq!(<&StaticCopyMap<bool, _>>::try_from(slice).unwrap(), &map);
    assert_eq!(<StaticCopyMap<bool, _>>::try_from(slice).unwrap(), map);
    let slice: &mut [u8] = &mut [0, 1];
    assert_eq!(
        <&mut StaticCopyMap<bool, _>>::try_from(slice).unwrap(),
        &mut map
    );
    let slice: &mut [u8] = &mut [0, 1];
    assert_eq!(<StaticCopyMap<bool, _>>::try_from(slice).unwrap(), map);
    let vec = vec![0, 1];
    assert_eq!(<StaticCopyMap<bool, _>>::try_from(vec).unwrap(), map);

    let slice: &[u8] = &[0];
    assert!(<&StaticCopyMap<bool, _>>::try_from(slice).is_err());
    assert!(<StaticCopyMap<bool, _>>::try_from(slice).is_err());
    let slice: &mut [u8] = &mut [0];
    assert!(<&mut StaticCopyMap<bool, _>>::try_from(slice).is_err());
    let slice: &mut [u8] = &mut [0];
    assert!(<StaticCopyMap<bool, _>>::try_from(slice).is_err());
    let vec = vec![0];
    assert!(<StaticCopyMap<bool, _>>::try_from(vec).is_err());

    let slice: &[u8] = &[0, 1, 2];
    assert!(<&StaticCopyMap<bool, _>>::try_from(slice).is_err());
    assert!(<StaticCopyMap<bool, _>>::try_from(slice).is_err());
    let slice: &mut [u8] = &mut [0, 1, 2];
    assert!(<&mut StaticCopyMap<bool, _>>::try_from(slice).is_err());
    let slice: &mut [u8] = &mut [0, 1, 2];
    assert!(<StaticCopyMap<bool, _>>::try_from(slice).is_err());
    let vec = vec![0, 1, 2];
    assert!(<StaticCopyMap<bool, _>>::try_from(vec).is_err());
}

#[test]
fn from_iterator() {
    let array: [(bool, u8); 0] = [];
    let map: StaticCopyMap<bool, u8> = array.into_iter().collect();
    assert_eq!(map[false], 0);
    assert_eq!(map[true], 0);
    let array: [(&bool, u8); 0] = [];
    let map: StaticCopyMap<bool, u8> = array.into_iter().collect();
    assert_eq!(map[false], 0);
    assert_eq!(map[true], 0);
    let map: StaticCopyMap<bool, u8> = [(false, 1)].into_iter().collect();
    assert_eq!(map[false], 1);
    assert_eq!(map[true], 0);
    let map: StaticCopyMap<bool, u8> = [(&false, 1)].into_iter().collect();
    assert_eq!(map[false], 1);
    assert_eq!(map[true], 0);
    let map: StaticCopyMap<bool, u8> = [(true, 2)].into_iter().collect();
    assert_eq!(map[false], 0);
    assert_eq!(map[true], 2);
    let map: StaticCopyMap<bool, u8> = [(&true, 2)].into_iter().collect();
    assert_eq!(map[false], 0);
    assert_eq!(map[true], 2);
    let map: StaticCopyMap<bool, u8> = [(false, 1), (true, 2)].into_iter().collect();
    assert_eq!(map[false], 1);
    assert_eq!(map[true], 2);
    let map: StaticCopyMap<bool, u8> = [(&false, 1), (&true, 2)].into_iter().collect();
    assert_eq!(map[false], 1);
    assert_eq!(map[true], 2);
}
