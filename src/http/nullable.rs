use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// A field of a partial update that can be left out, sent as `null` to clear
/// it, or sent with a value. `Option` alone cannot tell the first two apart.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum Nullable<T> {
    /// Not in the request: leave the stored value alone.
    #[default]
    Missing,
    /// Sent as `null`: clear the stored value.
    Null,
    Value(T),
}

impl<T> Nullable<T> {
    /// `None` when left out, `Some(None)` to clear, `Some(Some(v))` to set.
    pub fn into_change(self) -> Option<Option<T>> {
        match self {
            Self::Missing => None,
            Self::Null => Some(None),
            Self::Value(value) => Some(Some(value)),
        }
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Nullable<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(Option::<T>::deserialize(deserializer)?.map_or(Self::Null, Self::Value))
    }
}

impl<T: Serialize> Serialize for Nullable<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Value(value) => serializer.serialize_some(value),
            Self::Missing | Self::Null => serializer.serialize_none(),
        }
    }
}
