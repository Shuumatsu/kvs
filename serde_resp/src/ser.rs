use std::io::Write;

use serde::ser::*;

use crate::error::{Error, Result};
use crate::Resp;

impl Serialize for Resp {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Resp::SimpleString(str) => serializer.serialize_str(&("+".to_owned() + str)),
            Resp::Error(str) => serializer.serialize_str(&("-".to_owned() + str)),
            Resp::Integer(i) => serializer.serialize_i64(*i),
            Resp::BulkString(bulk_str) => match bulk_str {
                None => serializer.serialize_none(),
                Some(val) => serializer.serialize_bytes(val),
            },
            Resp::Array(arr) => match arr {
                None => serializer.serialize_unit(),
                Some(vals) => {
                    let mut s = serializer.serialize_seq(Some(vals.len()))?;
                    for v in vals {
                        s.serialize_element(v)?;
                    }
                    s.end()
                }
            },
        }
    }
}

pub struct Runner<W: Write> {
    writer: W,
}

pub fn to_writer(value: &impl Serialize, writer: &mut impl Write) -> Result<()> {
    let mut serializer = Runner { writer };
    value.serialize(&mut serializer)?;
    Ok(())
}

pub fn to_string(value: &impl Serialize) -> Result<String> {
    let mut buf: Vec<u8> = vec![];
    to_writer(value, &mut buf)?;
    Ok(String::from_utf8(buf)?)
}

impl<'a, W: Write> SerializeSeq for &'a mut Runner<W> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    // Close the sequence.
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, W: Write> SerializeTuple for &'a mut Runner<W> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unreachable!()
    }

    // Close the sequence.
    fn end(self) -> Result<()> {
        unreachable!()
    }
}

impl<'a, W: Write> SerializeTupleStruct for &'a mut Runner<W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unreachable!()
    }

    // Close the sequence.
    fn end(self) -> Result<()> {
        unreachable!()
    }
}

impl<'a, W: Write> SerializeTupleVariant for &'a mut Runner<W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unreachable!()
    }

    // Close the sequence.
    fn end(self) -> Result<()> {
        unreachable!()
    }
}

impl<'a, W: Write> SerializeMap for &'a mut Runner<W> {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unreachable!()
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unreachable!()
    }

    // Close the sequence.
    fn end(self) -> Result<()> {
        unreachable!()
    }
}

impl<'a, W: Write> SerializeStruct for &'a mut Runner<W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, _key: &'static str, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unreachable!()
    }
    fn end(self) -> Result<()> {
        unreachable!()
    }
}

impl<'a, W: Write> SerializeStructVariant for &'a mut Runner<W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, _key: &'static str, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unreachable!()
    }
    fn end(self) -> Result<()> {
        unreachable!()
    }
}

impl<'a, W: Write> Serializer for &'a mut Runner<W> {
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_i64(self, v: i64) -> Result<()> {
        self.writer.write_all(b":")?;
        self.writer.write_all(v.to_string().as_bytes())?;
        self.writer.write_all(b"\r\n")?;
        Ok(())
    }

    // for SimpleString and Error
    fn serialize_str(self, v: &str) -> Result<()> {
        self.writer.write_all(v.as_bytes())?;
        self.writer.write_all(b"\r\n")?;
        Ok(())
    }

    // Serialize a byte array as an array of bytes. Could also use a base64
    // string here. Binary formats will typically represent byte arrays more
    // compactly.
    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        self.writer.write_all(b"$")?;
        self.writer.write_all(v.len().to_string().as_bytes())?;
        self.writer.write_all(b"\r\n")?;
        self.writer.write_all(v)?;
        self.writer.write_all(b"\r\n")?;
        Ok(())
    }

    // for NULL BulkString
    fn serialize_none(self) -> Result<()> {
        self.writer.write_all(b"$-1\r\n")?;
        Ok(())
    }

    // for NULL Array
    fn serialize_unit(self) -> Result<()> {
        self.writer.write_all(b"*-1\r\n")?;
        Ok(())
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        match len {
            None => unreachable!(),
            Some(len) => {
                self.writer.write_all(b"*")?;
                self.writer.write_all(len.to_string().as_bytes())?;
                self.writer.write_all(b"\r\n")?;
            }
        }
        Ok(self)
    }

    fn serialize_bool(self, v: bool) -> Result<()> {
        unreachable!()
    }
    fn serialize_i8(self, v: i8) -> Result<()> {
        unreachable!()
    }
    fn serialize_i16(self, v: i16) -> Result<()> {
        unreachable!()
    }
    fn serialize_i32(self, v: i32) -> Result<()> {
        unreachable!()
    }
    fn serialize_u8(self, v: u8) -> Result<()> {
        unreachable!()
    }
    fn serialize_u16(self, v: u16) -> Result<()> {
        unreachable!()
    }
    fn serialize_u32(self, v: u32) -> Result<()> {
        unreachable!()
    }
    fn serialize_u64(self, v: u64) -> Result<()> {
        unreachable!()
    }
    fn serialize_f32(self, v: f32) -> Result<()> {
        unreachable!()
    }
    fn serialize_f64(self, v: f64) -> Result<()> {
        unreachable!()
    }

    fn serialize_char(self, v: char) -> Result<()> {
        unreachable!()
    }
    fn serialize_some<T>(self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unreachable!()
    }
    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        unreachable!()
    }
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        unreachable!()
    }
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        unreachable!()
    }
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        unreachable!()
    }
    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        unreachable!()
    }
    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        unreachable!()
    }
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<()> {
        unreachable!()
    }
    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unreachable!()
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unreachable!()
    }
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        unreachable!()
    }
}
