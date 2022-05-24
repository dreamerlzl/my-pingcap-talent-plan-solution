use std::ops::{AddAssign, MulAssign};
use std::str::FromStr;

use serde::de::{self, Visitor};
use serde::Deserialize;

use crate::error::{Error, Result};
use crate::visitor::RESPSeqAccess;

macro_rules! unsupported_type {
    ($($fn:ident),*) => {
        $(
            fn $fn<V>(self, _visitor: V) -> Result<V::Value>
            where V: Visitor<'de>, {
                unimplemented!()
            }
        )*
    }
}

pub struct RESPDeserializer<'de> {
    input: &'de [u8],
}

impl<'de> RESPDeserializer<'de> {
    pub fn from_bytes(input: &'de [u8]) -> Self {
        RESPDeserializer { input }
    }

    fn peek_byte(&mut self) -> Result<u8> {
        if let Some(byte) = self.input.iter().next() {
            return Ok(*byte);
        }
        Err(Error::Eof)
    }

    fn next_byte(&mut self) -> Result<u8> {
        let byte = self.peek_byte()?;
        self.input = &self.input[1..];
        Ok(byte)
    }

    fn trim_crlf(&mut self) -> Result<()> {
        if let Ok(b'\r') = self.next_byte() {
            if let Ok(b'\n') = self.next_byte() {
                Ok(())
            } else {
                Err(Error::ExpectedLF)
            }
        } else {
            Err(Error::ExpectedCR)
        }
    }

    fn parse_string(&mut self) -> Result<&'de [u8]> {
        match self.input.iter().position(|&r| r == b'\r') {
            Some(len) => {
                let s = &self.input[..len];
                self.input = &self.input[len..];
                Ok(s)
            }
            None => Err(Error::ExpectedCR),
        }
    }

    fn parse_string_with_size_hint(&mut self, len: usize) -> Result<&'de [u8]> {
        if len > self.input.len() {
            return Err(Error::Message(format!(
                "size hint {} larger than actual size {}!",
                len,
                self.input.len()
            )));
        }
        let s = &self.input[..len];
        self.input = &self.input[len..];
        Ok(s)
    }

    fn parse_signed<T>(&mut self) -> Result<T>
    where
        T: FromStr,
    {
        let mut bytes = Vec::new();
        if self.peek_byte()? == b'-' {
            bytes.push(self.next_byte()?);
        }
        while let Ok(_ch @ b'0'..=b'9') = self.peek_byte() {
            bytes.push(self.next_byte()?);
        }
        if bytes.is_empty() {
            return Err(Error::ExpectedInteger);
        }
        let int_str = String::from_utf8(bytes).map_err(|_| Error::ExpectedInteger)?;
        let int = int_str.parse::<T>().map_err(|_| Error::ExpectedInteger)?;
        Ok(int)
    }

    fn parse_unsigned<T>(&mut self) -> Result<T>
    where
        T: AddAssign<T> + MulAssign<T> + From<u8>,
    {
        let mut int = match self.next_byte()? {
            ch @ b'0'..=b'9' => T::from(ch as u8 - b'0'),
            _ => {
                return Err(Error::ExpectedInteger);
            }
        };
        while let Ok(_ch @ b'0'..=b'9') = self.peek_byte() {
            int *= T::from(10);
            int += T::from(self.next_byte()? as u8 - b'0');
        }
        Ok(int)
    }
}

pub fn from_bytes<'a, T>(input: &'a [u8]) -> Result<T>
where
    T: Deserialize<'a>,
{
    let mut deserializer = RESPDeserializer::from_bytes(input);
    let t = T::deserialize(&mut deserializer)?;
    if deserializer.input.is_empty() {
        Ok(t)
    } else {
        Err(Error::TrailingCharacters)
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut RESPDeserializer<'de> {
    type Error = Error;

    unsupported_type!(
        deserialize_i8,
        deserialize_i16,
        deserialize_i32,
        deserialize_u8,
        deserialize_u16,
        deserialize_u32,
        deserialize_u64,
        deserialize_unit,
        deserialize_char,
        deserialize_f32,
        deserialize_f64,
        deserialize_bytes,
        deserialize_byte_buf,
        deserialize_map,
        deserialize_identifier,
        deserialize_bool
    );

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.next_byte()? {
            b'+' => self.deserialize_str(visitor),
            b'-' => self.deserialize_string(visitor),
            b':' => self.deserialize_i64(visitor),
            b'$' => self.deserialize_option(visitor),
            b'*' => self.deserialize_seq(visitor),
            _ => Err(Error::Syntax),
        }
    }
    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let val = self.parse_signed()?;
        self.trim_crlf()?;
        visitor.visit_i64(val)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let val = self.parse_string()?;
        let val = String::from_utf8_lossy(val);
        self.trim_crlf()?;
        visitor.visit_string(val.to_string())
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let val = self.parse_string()?;
        let val = std::str::from_utf8(val)
            .map_err(|_| Error::Message("invalid utf-8 in safe string".to_owned()))?;
        self.trim_crlf()?;
        visitor.visit_str(val)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let num_bytes: i64 = self.parse_signed()?;
        self.trim_crlf()?;
        if num_bytes >= 0 {
            let val = self.parse_string_with_size_hint(num_bytes as usize)?;
            self.trim_crlf()?;
            visitor.visit_byte_buf(Vec::from(val))
        } else {
            visitor.visit_none()
        }
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let num_elem: usize = self.parse_unsigned()?;
        self.trim_crlf()?;
        visitor.visit_seq(RESPSeqAccess::new(self, num_elem))
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_tuple<V>(self, _len: usize, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        _visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Error, RESP};

    #[test]
    fn test1() {
        let cases = [
            ("123", 123),
            ("-12", -12),
            ("9", 9),
            ("802", 802),
            ("-11", -11),
            ("0", 0),
            ("-0", 0),
        ];

        for (case, er) in cases.into_iter() {
            let case = case.as_bytes();
            let mut deserializer = RESPDeserializer::from_bytes(case);
            assert_eq!(deserializer.parse_signed(), Ok(er));
        }
    }

    #[test]
    fn test2() {
        let cases = [
            (&i64::MAX.to_string(), i64::MAX),
            (&i64::MIN.to_string(), i64::MIN),
        ];

        for (case, er) in cases.into_iter() {
            let case = case.as_bytes();
            let mut deserializer = RESPDeserializer::from_bytes(case);
            assert_eq!(deserializer.parse_signed(), Ok(er));
        }
    }

    #[test]
    fn test_de() {
        let cases = [
            (
                "+OK\r\n".as_bytes(),
                Ok(RESP::SimpleString("OK".to_owned())),
            ),
            ("-Err\r\n".as_bytes(), Ok(RESP::Error("Err".to_owned()))),
            (":1231241421\r\n".as_bytes(), Ok(RESP::Integer(1231241421))),
            (":0\r\n".as_bytes(), Ok(RESP::Integer(0))),
            (":0".as_bytes(), Err(Error::ExpectedCR)),
            (":0\r".as_bytes(), Err(Error::ExpectedLF)),
            (":-1234\r\n".as_bytes(), Ok(RESP::Integer(-1234))),
            ("fuck".as_bytes(), Err(Error::Syntax)),
        ];

        for (case, er) in cases {
            assert_eq!(from_bytes(case), er);
        }
    }

    #[test]
    fn test_de_complex() {
        let cases = [
            (
                "$0\r\n\r\n".as_bytes(),
                Ok(RESP::BulkString(Some(Vec::new()))),
            ),
            ("$-1\r\n".as_bytes(), Ok(RESP::BulkString(None))),
            (
                "$5\r\nhello\r\n".as_bytes(),
                Ok(RESP::BulkString(Some(vec![b'h', b'e', b'l', b'l', b'o']))),
            ),
            ("*0\r\n".as_bytes(), Ok(RESP::Array(Vec::new()))),
            (
                "*2\r\n$5\r\nhello\r\n$5\r\nworld\r\n".as_bytes(),
                Ok(RESP::Array(vec![
                    RESP::BulkString(Some(vec![b'h', b'e', b'l', b'l', b'o'])),
                    RESP::BulkString(Some(vec![b'w', b'o', b'r', b'l', b'd'])),
                ])),
            ),
            (
                b"*5\r\n+OK\r\n-Error message\r\n:0\r\n$5\r\nhello\r\n*0\r\n",
                Ok(RESP::Array(vec![
                    RESP::SimpleString("OK".to_owned()),
                    RESP::Error("Error message".to_owned()),
                    RESP::Integer(0),
                    RESP::BulkString(Some(b"hello".to_vec())),
                    RESP::Array(Vec::new()),
                ])),
            ),
        ];

        for (case, er) in cases {
            assert_eq!(from_bytes(case), er);
        }
    }
}
