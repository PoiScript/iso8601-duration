use std::fmt;

use serde::{de::Error, de::Visitor, Deserialize, Deserializer, Serialize};

use crate::Duration;

impl Serialize for Duration {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

struct DurationVisitor;

impl<'de> Visitor<'de> for DurationVisitor {
    type Value = Duration;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an iso8601 duration format")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        v.parse().map_err(|err| E::custom(format!("{:?}", err)))
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: Error,
    {
        v.parse().map_err(|err| E::custom(format!("{:?}", err)))
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        v.parse().map_err(|err| E::custom(format!("{:?}", err)))
    }
}

impl<'de> Deserialize<'de> for Duration {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(DurationVisitor)
    }
}

#[test]
fn test_serde() {
    use serde_json::{from_str, to_string};

    let s = r#""P3Y6M4DT12H30M5S""#;

    let d: Duration = from_str(&s).unwrap();

    assert_eq!(to_string(&d).unwrap(), s);
}
