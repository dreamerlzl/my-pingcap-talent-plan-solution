use std::fmt;

use serde::de::{self, DeserializeSeed, SeqAccess, Visitor};
use serde::Deserialize;

use crate::de::RESPDeserializer;
use crate::{Error, RESP};

impl<'de> Deserialize<'de> for RESP {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(RESPVisitor)
    }
}

struct RESPVisitor;

impl<'de> Visitor<'de> for RESPVisitor {
    type Value = RESP;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a RESP type")
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(RESP::Integer(value))
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(RESP::Error(v))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(RESP::SimpleString(v.to_owned()))
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(RESP::BulkString(None))
    }

    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(RESP::BulkString(Some(v)))
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut ret = Vec::with_capacity(seq.size_hint().unwrap_or_default());
        while let Some(elem) = seq.next_element()? {
            ret.push(elem);
        }
        Ok(RESP::Array(ret))
    }
}

pub struct RESPSeqAccess<'a, 'de> {
    de: &'a mut RESPDeserializer<'de>,
    num_left_ele: usize,
}

impl<'a, 'de> RESPSeqAccess<'a, 'de> {
    pub fn new(de: &'a mut RESPDeserializer<'de>, num_left_ele: usize) -> Self {
        RESPSeqAccess { de, num_left_ele }
    }
}

impl<'a, 'de> SeqAccess<'de> for RESPSeqAccess<'a, 'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Error>
    where
        T: DeserializeSeed<'de>,
    {
        if self.num_left_ele == 0 {
            return Ok(None);
        }
        self.num_left_ele -= 1;
        seed.deserialize(&mut *self.de).map(Some)
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.num_left_ele)
    }
}
