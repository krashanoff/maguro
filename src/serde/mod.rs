//! Extensions to serde for deserializing foreign types.
//!
//! Provides deserializers for [Durations](std::time::Duration),
//! and for converting types such as [&str] to [u32](std::u32).

use serde::{
    de::{Error, Visitor},
    Deserialize, Deserializer,
};

pub mod mime {
    //! Extensions for serializing and deserializing [mime::Mime](::mime::Mime).

    use super::*;
    use ::mime;
    use serde::Serializer;
    use std::{fmt, str::FromStr};

    struct MimeOptionVisitor;

    impl<'de> Visitor<'de> for MimeOptionVisitor {
        type Value = Option<mime::Mime>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            write!(formatter, "a valid MIME type string")
        }

        fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            let s: String = Deserialize::deserialize(deserializer)?;
            Ok(Some(mime::Mime::from_str(s.as_str()).map_err(Error::custom)?))
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

    /// Deserialize an `Option<mime::Mime>` from a string.
    pub fn option_from_str<'de, D>(deserializer: D) -> Result<Option<mime::Mime>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(deserializer.deserialize_option(MimeOptionVisitor)?)
    }

    /// Serialize a [mime::Mime] to a string.
    pub fn to_str<S>(m: &mime::Mime, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        s.serialize_str(m.to_string().as_str())
    }

    pub fn option_to_str<S>(m: &Option<mime::Mime>, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match m {
            Some(m) => s.serialize_some(&m.to_string()),
            None => s.serialize_none(),
        }
    }
}

pub mod duration {
    //! Extensions for parsing [Durations](Duration) and their
    //! [Options](Option<T>) from strings.

    use super::*;
    use std::{
        fmt::{self, Display},
        time::Duration,
    };

    enum Unit {
        Seconds,
        Millis,
    }

    impl Display for Unit {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                f,
                "{}",
                match self {
                    &Unit::Seconds => "seconds",
                    &Unit::Millis => "milliseconds",
                }
            )
        }
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
            write!(formatter, "a duration in {}", self.units)
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
