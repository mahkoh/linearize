use {
    linearize::{static_copy_map, static_map, Linearize, StaticMap},
    serde::{Deserialize, Serialize},
    serde_json::json,
};

#[test]
fn unit() {
    let map = static_map! {
        false => 11,
        true => 22,
    };
    let value = serde_json::to_value(&map).unwrap();
    assert_eq!(
        value,
        json!({
            "false": 11,
            "true": 22,
        })
    );
    let new_map = serde_json::from_value(value).unwrap();
    assert_eq!(map, new_map);
}

#[test]
fn copy_map() {
    let map = static_copy_map! {
        false => 11,
        true => 22,
    };
    let value = serde_json::to_value(map).unwrap();
    assert_eq!(
        value,
        json!({
            "false": 11,
            "true": 22,
        })
    );
    let new_map = serde_json::from_value(value).unwrap();
    assert_eq!(map, new_map);
}

#[test]
fn ordering() {
    #[derive(Linearize, Serialize, Deserialize, Debug)]
    enum O {
        Less,
        Equal,
        Greater,
    }
    let map = static_map! {
        O::Less => 11,
        O::Equal => 22,
        O::Greater => 33,
    };
    let value = serde_json::to_value(&map).unwrap();
    assert_eq!(
        value,
        json!({
            "Less": 11,
            "Equal": 22,
            "Greater": 33,
        })
    );
    let new_map = serde_json::from_value(value).unwrap();
    assert_eq!(map, new_map);
}

#[test]
fn missing_key() {
    let value = json!({
        "false": 11,
    });
    let err = serde_json::from_value::<StaticMap<bool, u8>>(value).unwrap_err();
    assert!(
        err.to_string().contains("Missing key true in static map"),
        "{:?}",
        err
    );
}

#[test]
fn wrong_type() {
    let value = json!([11, 22]);
    let err = serde_json::from_value::<StaticMap<bool, u8>>(value).unwrap_err();
    assert!(err.to_string().contains("a map"), "{:?}", err);
}

#[test]
fn use_default() {
    let value = json!({
        "false": 11,
    });
    #[derive(Deserialize)]
    #[serde(transparent)]
    struct S {
        #[serde(deserialize_with = "linearize::serde_1::use_default::deserialize")]
        map: StaticMap<bool, u8>,
    }
    let map = serde_json::from_value::<S>(value).unwrap().map;
    assert_eq!(map[false], 11);
    assert_eq!(map[true], 0);
}

#[test]
fn skip_none() {
    #[derive(Serialize, Deserialize)]
    #[serde(transparent)]
    struct S {
        #[serde(with = "linearize::serde_1::skip_none")]
        map: StaticMap<bool, Option<u8>>,
    }
    let s = S {
        map: static_map! {
            false => Some(11),
            true => None,
        },
    };
    let value = serde_json::to_value(&s).unwrap();
    assert_eq!(
        value,
        json!({
            "false": 11,
        })
    );
    let map = serde_json::from_value::<S>(value).unwrap().map;
    assert_eq!(map[false], Some(11));
    assert_eq!(map[true], None);
}
