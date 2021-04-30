//! Extensions to serde for deserializing unsupported types.
//!
//! Provides deserializers for [Duration]s, and for converting
//! types such as [&str] to [u32], or [&str] to [mime::Mime].

pub mod duration {
    //! Extensions for parsing [Duration]s and their [Option<T>]s from
    //! strings.

    use serde::{
        de::{Error, Visitor},
        Deserialize, Deserializer,
    };
    use std::{fmt, str, time::Duration};

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

pub mod u32 {
    //! Extensions for parsing `u32` and `Option<u32>` from string types.

    use serde::{
        de::{Error, Visitor},
        Deserialize, Deserializer,
    };
    use std::{fmt, str};

    struct U32OptionVisitor;

    impl<'de> Visitor<'de> for U32OptionVisitor {
        type Value = Option<u32>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            write!(formatter, "a valid u32 integer")
        }

        fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            let s: &str = Deserialize::deserialize(deserializer)?;
            Ok(Some(s.parse().map_err(D::Error::custom)?))
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

    pub fn from_str<'de, D>(d: D) -> Result<u32, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(d)?;
        Ok(s.parse().map_err(D::Error::custom)?)
    }

    pub fn from_str_option<'de, D>(d: D) -> Result<Option<u32>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(d.deserialize_option(U32OptionVisitor)?)
    }
}
