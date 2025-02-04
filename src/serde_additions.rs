pub mod unix_timestamp {
    use std::fmt;

    use chrono::{DateTime, TimeZone, Utc};
    use serde::{
        de::{Error as DeError, Visitor},
        Deserializer, Serializer,
    };

    struct TimestampVisitor;

    impl<'de> Visitor<'de> for TimestampVisitor {
        type Value = DateTime<Utc>;

        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where
            E: DeError,
        {
            Ok(Utc.timestamp(value, 0))
        }

        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: DeError,
        {
            Ok(Utc.timestamp(value as _, 0))
        }

        fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
        where
            E: DeError,
        {
            Ok(Utc.timestamp(value as _, 0))
        }

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("Unix timestamp")
        }
    }

    pub fn serialize<S: Serializer>(
        time: &Option<DateTime<Utc>>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.serialize_i64(time.unwrap().timestamp())
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Option<DateTime<Utc>>, D::Error> {
        deserializer.deserialize_i64(TimestampVisitor).map(Some)
    }
}

pub mod audiences {
    use std::collections::HashSet;
    use std::fmt;

    use serde::{
        de::{Error as DeError, SeqAccess, Visitor},
        ser::SerializeSeq,
        Deserializer, Serialize, Serializer,
    };

    use super::super::claims::Audiences;

    struct AudiencesVisitor;

    impl<'de> Visitor<'de> for AudiencesVisitor {
        type Value = Audiences;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("Audiences")
        }

        fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
        where
            E: DeError,
        {
            Ok(Audiences::AsString(value))
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: DeError,
        {
            Ok(Audiences::AsString(value.to_string()))
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let mut audiences_set: HashSet<String> =
                HashSet::with_capacity(seq.size_hint().unwrap_or(1));
            while let Some(audience) = seq.next_element()? {
                audiences_set.insert(audience);
            }
            Ok(Audiences::AsSet(audiences_set))
        }
    }

    pub fn serialize<S: Serializer>(
        audiences: &Option<Audiences>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        match audiences {
            None => serializer.serialize_seq(Some(0))?.end(),
            Some(Audiences::AsString(audience)) => audience.serialize(serializer),
            Some(Audiences::AsSet(audiences)) => audiences.serialize(serializer),
        }
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Option<Audiences>, D::Error> {
        let audiences = deserializer.deserialize_any(AudiencesVisitor)?;
        Ok(Some(audiences))
    }
}
