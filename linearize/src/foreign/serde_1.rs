//! The [`serde`][serde_1] implementations for this crate.
//!
//! The default implementations for `StaticMap<L, T>` use the same wire format as
//! `HashMap<L, T>`. If a key is missing during deserialization, the entire operation
//! fails.
//!
//! This behavior can be adjusted by using the [`skip_none`] and [`use_default`] modules.

mod default {
    use {
        crate::{Linearize, StaticCopyMap, StaticMap},
        core::{
            fmt::{Debug, Display, Formatter},
            marker::PhantomData,
            ops::Deref,
        },
        serde_1::{
            de::{Error, MapAccess, Visitor},
            ser::SerializeMap,
            Deserialize, Deserializer, Serialize, Serializer,
        },
    };

    impl<L, T> Serialize for StaticMap<L, T>
    where
        L: Linearize + Serialize,
        T: Serialize,
    {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let mut map = serializer.serialize_map(Some(L::LENGTH))?;
            for (k, v) in self {
                map.serialize_entry(&k, v)?;
            }
            map.end()
        }
    }

    impl<'de, L, T> Deserialize<'de> for StaticMap<L, T>
    where
        L: Linearize + Debug + Deserialize<'de>,
        T: Deserialize<'de>,
    {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_map(StaticMapVisitor(PhantomData))
        }
    }

    struct StaticMapVisitor<L, T>(PhantomData<fn() -> StaticMap<L, T>>)
    where
        L: Linearize;

    impl<'de, L, T> Visitor<'de> for StaticMapVisitor<L, T>
    where
        L: Linearize + Debug + Deserialize<'de>,
        T: Deserialize<'de>,
    {
        type Value = StaticMap<L, T>;

        fn expecting(&self, formatter: &mut Formatter) -> core::fmt::Result {
            write!(formatter, "a map")
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            let mut res = StaticMap::<L, Option<T>>::default();
            while let Some((k, v)) = map.next_entry::<L, T>()? {
                res[k] = Some(v);
            }

            let mut res = res.into_values();
            Ok(crate::static_map! { k => {
                res.next().flatten().ok_or(Error::custom(MissingKey(k)))?
            }})
        }
    }

    struct MissingKey<L>(L);
    impl<L: Debug> Display for MissingKey<L> {
        fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
            write!(f, "Missing key {:?} in static map", self.0)
        }
    }

    impl<L, T> Serialize for StaticCopyMap<L, T>
    where
        L: Linearize + Serialize,
        T: Copy + Serialize,
    {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            self.deref().serialize(serializer)
        }
    }

    impl<'de, L, T> Deserialize<'de> for StaticCopyMap<L, T>
    where
        L: Linearize + Debug + Deserialize<'de>,
        T: Copy + Deserialize<'de>,
    {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            StaticMap::deserialize(deserializer).map(|v| v.into_copy())
        }
    }
}

/// A de/serialize implementation for `StaticMap<L, Option<T>>` that skips `None` values.
///
/// The serializer skips entries whose value is `None`.
///
/// The deserializer uses `None` for missing entries.
///
/// The wire format is that of `HashMap<L, T>`, i.e. with the `Option` removed.
///
/// # Example
///
/// ```rust
/// # use serde_1::{Serialize, Deserialize};
/// # use linearize::StaticMap;
/// #[derive(Serialize, Deserialize)]
/// # #[serde(crate = "serde_1")]
/// struct X {
///     #[serde(with = "linearize::serde_1::skip_none")]
///     map: StaticMap<u8, Option<String>>,
/// }
/// ```
pub mod skip_none {
    use {
        crate::{Linearize, StaticMap},
        core::{fmt::Formatter, marker::PhantomData},
        serde_1::{
            de::{MapAccess, Visitor},
            ser::SerializeMap,
            Deserialize, Deserializer, Serialize, Serializer,
        },
    };

    pub fn serialize<L, T, S>(
        static_map: &StaticMap<L, Option<T>>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        L: Linearize + Serialize,
        T: Serialize,
        S: Serializer,
    {
        let count = static_map.values().filter(|v| v.is_some()).count();
        let mut map = serializer.serialize_map(Some(count))?;
        for (k, v) in static_map {
            if let Some(v) = v {
                map.serialize_entry(&k, v)?;
            }
        }
        map.end()
    }

    pub fn deserialize<'de, L, T, D, O>(deserializer: D) -> Result<O, D::Error>
    where
        L: Deserialize<'de> + Linearize,
        T: Deserialize<'de>,
        D: Deserializer<'de>,
        O: From<StaticMap<L, Option<T>>>,
    {
        deserializer
            .deserialize_map(V(PhantomData))
            .map(|v| v.into())
    }

    struct V<L, T>(PhantomData<fn() -> StaticMap<L, T>>)
    where
        L: Linearize;

    impl<'de, L, T> Visitor<'de> for V<L, T>
    where
        L: Deserialize<'de> + Linearize,
        T: Deserialize<'de>,
    {
        type Value = StaticMap<L, Option<T>>;

        fn expecting(&self, formatter: &mut Formatter) -> core::fmt::Result {
            write!(formatter, "a map")
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            let mut res = StaticMap::<L, Option<T>>::default();
            while let Some((k, v)) = map.next_entry::<L, T>()? {
                res[k] = Some(v);
            }
            Ok(res)
        }
    }
}

/// A deserialize implementation replaces missing values by the default.
///
/// # Example
///
/// ```rust
/// # use serde_1::Deserialize;
/// # use linearize::StaticMap;
/// #[derive(Deserialize)]
/// # #[serde(crate = "serde_1")]
/// struct X {
///     #[serde(deserialize_with = "linearize::serde_1::use_default::deserialize")]
///     map: StaticMap<u8, String>,
/// }
/// ```
pub mod use_default {
    use {
        crate::{Linearize, StaticMap},
        core::{fmt::Formatter, marker::PhantomData},
        serde_1::{
            de::{MapAccess, Visitor},
            Deserialize, Deserializer,
        },
    };

    pub fn deserialize<'de, L, T, D, O>(deserializer: D) -> Result<O, D::Error>
    where
        L: Deserialize<'de> + Linearize,
        T: Deserialize<'de> + Default,
        D: Deserializer<'de>,
        O: From<StaticMap<L, T>>,
    {
        deserializer
            .deserialize_map(V(PhantomData))
            .map(|v| v.into())
    }

    struct V<L, T>(PhantomData<fn() -> StaticMap<L, T>>)
    where
        L: Linearize;

    impl<'de, L, T> Visitor<'de> for V<L, T>
    where
        L: Deserialize<'de> + Linearize,
        T: Deserialize<'de> + Default,
    {
        type Value = StaticMap<L, T>;

        fn expecting(&self, formatter: &mut Formatter) -> core::fmt::Result {
            write!(formatter, "a map")
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            let mut res = StaticMap::<L, Option<T>>::default();
            while let Some((k, v)) = map.next_entry::<L, T>()? {
                res[k] = Some(v);
            }
            Ok(res.map_values(|v| v.unwrap_or_default()))
        }
    }
}
