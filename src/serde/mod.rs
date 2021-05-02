//! Extensions to serde for deserializing foreign types.
//!
//! Provides deserializers for [Durations](std::time::Duration),
//! and for converting types such as [&str] to [u32](std::u32).

use serde::{
    de::{Error, Visitor},
    Deserialize, Deserializer, Serializer,
};

pub mod mime {
    //! Extensions for serializing and deserializing [mime::Mime](::mime::Mime).

    use super::*;
    use ::mime;
    use std::str::FromStr;

    pub fn to_mime<'de, D>(d: D) -> Result<mime::Mime, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(mime::Mime::from_str(Deserialize::deserialize(d)?).map_err(Error::custom)?)
    }

    pub fn to_str<S>(m: &mime::Mime, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        s.serialize_str(m.to_string().as_str())
    }
}

pub mod duration {
    //! Extensions for parsing [Durations](Duration) and their
    //! [Options](Option<T>) from strings.

    use super::*;
    use std::{fmt, time::Duration};

    enum Unit {
        Seconds,
        Millis,
    }

    struct DurationOptionVisitor {
        units: Unit,
    }

    impl DurationOptionVisitor {
        fn new(u: Unit) -> Self {
            Self { units: u }
        }
    }

    impl<'de> Visitor<'de> for DurationOptionVisitor {
        type Value = Option<Duration>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            write!(formatter, "a duration in seconds")
        }

        fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            let s: &str = Deserialize::deserialize(deserializer)?;

            Ok(Some(match self.units {
                Unit::Millis => Duration::from_millis(s.parse().map_err(Error::custom)?),
                Unit::Seconds => Duration::from_secs(s.parse().map_err(Error::custom)?),
            }))
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: Error,
        {
            Ok(None)
        }

        fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: Error,
        {
            Ok(None)
        }
    }

    pub fn from_millis<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;
        Ok(Duration::from_millis(s.parse().map_err(D::Error::custom)?))
    }

    pub fn from_millis_option<'de, D>(deserializer: D) -> Result<Option<Duration>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(deserializer.deserialize_option(DurationOptionVisitor::new(Unit::Millis))?)
    }

    pub fn from_secs<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;
        Ok(Duration::from_secs(s.parse().map_err(D::Error::custom)?))
    }

    pub fn from_secs_option<'de, D>(deserializer: D) -> Result<Option<Duration>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(deserializer.deserialize_option(DurationOptionVisitor::new(Unit::Seconds))?)
    }
}
