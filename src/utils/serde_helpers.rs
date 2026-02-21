use serde::{Deserialize, Deserializer};

/// Deserialize `Option<Option<T>>` from JSON so that:
/// - a missing / `#[serde(default)]` field yields `None`  (skip the column in the UPDATE)
/// - an explicit `null` in JSON yields `Some(None)`        (set the column to NULL)
/// - a real value yields `Some(Some(value))`               (set the column to that value)
pub fn double_option<'de, T, D>(de: D) -> Result<Option<Option<T>>, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    Option::<Option<T>>::deserialize(de)
}
