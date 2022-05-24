use serde::{ser, Serialize};

use crate::error::{Error, Result};

pub struct Serializer {
    output: Vec<u8>,
}

impl Serializer {
    fn append_string<T: Into<String>>(&mut self, s: T) {
        self.output.extend_from_slice(s.into().as_bytes());
    }
}

pub fn to_bytes<T>(value: &T) -> Result<Vec<u8>>
where
    T: Serialize,
{
    let mut serializer = Serializer { output: Vec::new() };
    value.serialize(&mut serializer)?;
    Ok(serializer.output)
}

macro_rules! unsupported_type {
    ($($ty:ty, $fname: ident),*) => {
        $(
            fn $fname (self, _v: $ty) -> Result<()> {
                Err(Error::UnsupportedType)
            }
        )*
    }
}

macro_rules! ser_delegate {
    ($dty:ty, $dfname: ident, $($ty:ty, $fname: ident),*) => {
        $(
            fn $fname (self, v: $ty) -> Result<()> {
                self.$dfname(<$dty>::from(v))
            }
        )*
    }
}

impl<'a> ser::Serializer for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    unsupported_type!(
        f32,
        serialize_f32,
        f64,
        serialize_f64,
        char,
        serialize_char,
        &[u8],
        serialize_bytes
    );
    ser_delegate!(u64, serialize_u64, u16, serialize_u16, u32, serialize_u32);
    ser_delegate!(
        i64,
        serialize_i64,
        i8,
        serialize_i8,
        i16,
        serialize_i16,
        i32,
        serialize_i32
    );

    fn serialize_bool(self, v: bool) -> Result<()> {
        let vs = if v { "true" } else { "false" };
        self.append_string(vs);
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        self.output.push(v);
        Ok(())
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        self.output.extend_from_slice(v.as_bytes());
        Ok(())
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        self.append_string(v.to_string());
        Ok(())
    }

    fn serialize_i64(self, v: i64) -> Result<()> {
        self.append_string(v.to_string());
        Ok(())
    }

    fn serialize_none(self) -> Result<()> {
        self.serialize_unit()
    }

    fn serialize_unit(self) -> Result<()> {
        self.output.extend_from_slice("-1".as_bytes());
        Ok(())
    }

    fn serialize_some<T>(self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        self.serialize_unit()
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<()> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<()>
    where
        T: Serialize,
    {
        if name != "RESP" {
            return Err(Error::UnsupportedType);
        }
        if variant == "Array" {
            self.output.push(b'*');
            value.serialize(&mut *self)?;
        } else {
            let first = match variant {
                "SimpleString" => b'+',
                "BulkString" => b'$',
                "Error" => b'-',
                "Integer" => b':',
                _ => return Err(Error::Message("Unexpected RESP variant".to_owned())),
            };
            self.output.push(first);
            value.serialize(&mut *self)?;
            self.output.push(b'\r');
            self.output.push(b'\n');
        }
        Ok(())
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        if let Some(l) = len {
            self.append_string(l.to_string());
        } else {
            self.append_string("0");
        }
        self.output.push(b'\r');
        self.output.push(b'\n');
        // dbg!(&self.output);
        Ok(self)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Err(Error::UnsupportedType)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Err(Error::UnsupportedType)
    }

    // for struct Msg(i32, f32)
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Err(Error::UnsupportedType)
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Err(Error::UnsupportedType)
    }

    // for enum E { Msg {a: i32, b: i32} }
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Err(Error::UnsupportedType)
    }

    // for enum E { T(u8, u8), U(a, b, c) }
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Err(Error::UnsupportedType)
    }

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;
}

impl<'a> ser::SerializeSeq for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)?;
        Ok(())
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

macro_rules! unsupported_compound_type {
    ($($ty:ty, $fname: ident),*) => {
        $(
            impl<'a> $ty for &'a mut Serializer {
                type Ok = ();
                type Error = Error;

                fn $fname<T>(&mut self, _value: &T) -> Result<()>
                    where T: ?Sized + Serialize
                {
                    Err(Error::UnsupportedType)
                }

                fn end(self) -> Result<()> {
                    Err(Error::UnsupportedType)
                }
            }
        )*
    }
}

unsupported_compound_type!(
    ser::SerializeTuple,
    serialize_element,
    ser::SerializeTupleStruct,
    serialize_field,
    ser::SerializeTupleVariant,
    serialize_field
);

impl<'a> ser::SerializeStructVariant for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, _key: &'static str, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        Err(Error::UnsupportedType)
    }

    fn end(self) -> Result<()> {
        Err(Error::UnsupportedType)
    }
}

impl<'a> ser::SerializeStruct for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, _key: &'static str, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        Err(Error::UnsupportedType)
    }

    fn end(self) -> Result<()> {
        Err(Error::UnsupportedType)
    }
}

impl<'a> ser::SerializeMap for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    // The Serde data model allows map keys to be any serializable type. JSON
    // only allows string keys so the implementation below will produce invalid
    // JSON if the key serializes as something other than a string.
    //
    // A real JSON serializer would need to validate that map keys are strings.
    // This can be done by using a different Serializer to serialize the key
    // (instead of `&mut **self`) and having that other serializer only
    // implement `serialize_str` and return an error on any other data type.
    fn serialize_key<T>(&mut self, _key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        Err(Error::UnsupportedType)
    }

    // It doesn't make a difference whether the colon is printed at the end of
    // `serialize_key` or at the beginning of `serialize_value`. In this case
    // the code is a bit simpler having it here.
    fn serialize_value<T>(&mut self, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        Err(Error::UnsupportedType)
    }

    fn end(self) -> Result<()> {
        Err(Error::UnsupportedType)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RESP;

    #[test]
    fn test1() {
        let cases = [
            (RESP::SimpleString("OK".to_owned()), b"+OK\r\n".to_vec()),
            (
                RESP::Error("Error message".to_owned()),
                b"-Error message\r\n".to_vec(),
            ),
            (RESP::Integer(0), b":0\r\n".to_vec()),
            (RESP::Integer(1000), b":1000\r\n".to_vec()),
            (
                RESP::BulkString(Some(b"hello".to_vec())),
                b"$5\r\nhello\r\n".to_vec(),
            ),
            (RESP::BulkString(None), b"$-1\r\n".to_vec()),
            (RESP::BulkString(Some(b"".to_vec())), b"$0\r\n\r\n".to_vec()),
            (RESP::Array(Vec::new()), b"*0\r\n".to_vec()),
            (
                RESP::Array(vec![RESP::SimpleString("OK".to_owned())]),
                b"*1\r\n+OK\r\n".to_vec(),
            ),
            (
                RESP::Array(vec![
                    RESP::SimpleString("OK".to_owned()),
                    RESP::Error("Error message".to_owned()),
                    RESP::Integer(0),
                    RESP::BulkString(Some(b"hello".to_vec())),
                    RESP::Array(Vec::new()),
                ]),
                b"*5\r\n+OK\r\n-Error message\r\n:0\r\n$5\r\nhello\r\n*0\r\n".to_vec(),
            ),
            (
                RESP::Array(vec![
                    RESP::BulkString(Some(b"hello".to_vec())),
                    RESP::BulkString(None),
                    RESP::BulkString(Some(b"world".to_vec())),
                ]),
                b"*3\r\n$5\r\nhello\r\n$-1\r\n$5\r\nworld\r\n".to_vec(),
            ),
        ];
        for (case, er) in cases.into_iter() {
            let output = Ok(er);
            assert_eq!(to_bytes(&case), output);
        }
    }
}
