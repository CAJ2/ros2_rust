use std::fmt;

use serde::{de, Deserialize};

use crate::{BoundedSequenceValue, DynamicBoundedString, DynamicBoundedWString, MessageFieldInfo, SequenceValue};

use super::{DynamicMessageView, MessageStructure, Value, SimpleValue, ArrayValue, ValueKind, BaseType, Proxy as _};

impl serde::de::Error for super::DynamicMessageError {
    fn custom<T>(msg: T) -> Self
    where
        T: fmt::Display,
    {
        Self::SerdeMessage(msg.to_string())
    }
}

impl serde::ser::Error for super::DynamicMessageError {
    fn custom<T>(msg: T) -> Self
    where
        T: fmt::Display,
    {
        Self::SerdeMessage(msg.to_string())
    }
}

type Result<T> = std::result::Result<T, super::DynamicMessageError>;

/// A deserializer for dynamic messages.
#[derive(Debug)]
pub struct DynReader<'de> {
    structure: &'de MessageStructure,
    storage: &'de [u8],
    // (Current index, length, is a sequence?)
    pos: Vec<(usize, usize, bool)>,
    // Current field being read
    current: &'de MessageFieldInfo,
    // Offset of current within the storage bytes
    offset: usize
}

impl<'de> DynReader<'de> {
    pub(super) fn from_dyn_msg_view(view: DynamicMessageView<'de>) -> Self {
        Self {
            structure: view.structure,
            storage: view.storage,
            pos: Vec::new(),
            current: view.structure.fields.first().expect("Message has at least one field"),
            offset: 0,
        }
    }

    fn get_value(&self) -> Result<Value<'de>> {
        let size = self.current.size().unwrap_or(1);
        let offset = self.offset + self.current.offset;
        // SAFETY: The byte slice contains a valid field as assured by DynamicMessage construction
        let value = unsafe { Value::new(&self.storage[offset..offset + size], self.current) };
        let value = value.expect("Field value creation always returns Some");
        let pos = *self.pos.last().expect("pos stack is never empty");
        let idx = pos.0;
        match value {
            Value::Simple(val) => Ok(Value::Simple(val)),
            Value::Array(ArrayValue::BooleanArray(v)) => Ok(Value::Simple(SimpleValue::Boolean(&v[idx]))),
            Value::Array(ArrayValue::Int8Array(v)) => Ok(Value::Simple(SimpleValue::Int8(&v[idx]))),
            Value::Array(ArrayValue::Uint8Array(v)) => Ok(Value::Simple(SimpleValue::Uint8(&v[idx]))),
            Value::Array(ArrayValue::Int16Array(v)) => Ok(Value::Simple(SimpleValue::Int16(&v[idx]))),
            Value::Array(ArrayValue::Uint16Array(v)) => Ok(Value::Simple(SimpleValue::Uint16(&v[idx]))),
            Value::Array(ArrayValue::Int32Array(v)) => Ok(Value::Simple(SimpleValue::Int32(&v[idx]))),
            Value::Array(ArrayValue::Uint32Array(v)) => Ok(Value::Simple(SimpleValue::Uint32(&v[idx]))),
            Value::Array(ArrayValue::Int64Array(v)) => Ok(Value::Simple(SimpleValue::Int64(&v[idx]))),
            Value::Array(ArrayValue::Uint64Array(v)) => Ok(Value::Simple(SimpleValue::Uint64(&v[idx]))),
            Value::Array(ArrayValue::FloatArray(v)) => Ok(Value::Simple(SimpleValue::Float(&v[idx]))),
            Value::Array(ArrayValue::DoubleArray(v)) => Ok(Value::Simple(SimpleValue::Double(&v[idx]))),
            Value::Array(ArrayValue::StringArray(v)) => Ok(Value::Simple(SimpleValue::String(&v[idx]))),
            Value::Array(ArrayValue::WStringArray(v)) => Ok(Value::Simple(SimpleValue::WString(&v[idx]))),
            Value::Array(ArrayValue::CharArray(v)) => Ok(Value::Simple(SimpleValue::Char(&v[idx]))),
            Value::Array(ArrayValue::WCharArray(v)) => Ok(Value::Simple(SimpleValue::WChar(&v[idx]))),
            Value::Array(ArrayValue::OctetArray(v)) => Ok(Value::Simple(SimpleValue::Octet(&v[idx]))),
            Value::Array(ArrayValue::BoundedStringArray(v)) => {
                let size = self.current.base_type.size().unwrap_or(1);
                let start = idx * size + offset;
                let bytes = &self.storage[start..start + size];
                unsafe { Ok(Value::Simple(SimpleValue::BoundedString(DynamicBoundedString::new(bytes, v[idx].upper_bound())))) }
            },
            Value::Array(ArrayValue::BoundedWStringArray(v)) => {
                let size = self.current.base_type.size().unwrap_or(1);
                let start = idx * size + offset;
                let bytes = &self.storage[start..start + size];
                unsafe { Ok(Value::Simple(SimpleValue::BoundedWString(DynamicBoundedWString::new(bytes, v[idx].upper_bound())))) }
            },
            Value::Sequence(SequenceValue::BooleanSequence(v)) => Ok(Value::Simple(SimpleValue::Boolean(&v[idx]))),
            Value::Sequence(SequenceValue::Int8Sequence(v)) => Ok(Value::Simple(SimpleValue::Int8(&v[idx]))),
            Value::Sequence(SequenceValue::Uint8Sequence(v)) => Ok(Value::Simple(SimpleValue::Uint8(&v[idx]))),
            Value::Sequence(SequenceValue::Int16Sequence(v)) => Ok(Value::Simple(SimpleValue::Int16(&v[idx]))),
            Value::Sequence(SequenceValue::Uint16Sequence(v)) => Ok(Value::Simple(SimpleValue::Uint16(&v[idx]))),
            Value::Sequence(SequenceValue::Int32Sequence(v)) => Ok(Value::Simple(SimpleValue::Int32(&v[idx]))),
            Value::Sequence(SequenceValue::Uint32Sequence(v)) => Ok(Value::Simple(SimpleValue::Uint32(&v[idx]))),
            Value::Sequence(SequenceValue::Int64Sequence(v)) => Ok(Value::Simple(SimpleValue::Int64(&v[idx]))),
            Value::Sequence(SequenceValue::Uint64Sequence(v)) => Ok(Value::Simple(SimpleValue::Uint64(&v[idx]))),
            Value::Sequence(SequenceValue::FloatSequence(v)) => Ok(Value::Simple(SimpleValue::Float(&v[idx]))),
            Value::Sequence(SequenceValue::DoubleSequence(v)) => Ok(Value::Simple(SimpleValue::Double(&v[idx]))),
            Value::Sequence(SequenceValue::StringSequence(v)) => Ok(Value::Simple(SimpleValue::String(&v[idx]))),
            Value::Sequence(SequenceValue::WStringSequence(v)) => Ok(Value::Simple(SimpleValue::WString(&v[idx]))),
            Value::Sequence(SequenceValue::CharSequence(v)) => Ok(Value::Simple(SimpleValue::Char(&v[idx]))),
            Value::Sequence(SequenceValue::WCharSequence(v)) => Ok(Value::Simple(SimpleValue::WChar(&v[idx]))),
            Value::Sequence(SequenceValue::OctetSequence(v)) => Ok(Value::Simple(SimpleValue::Octet(&v[idx]))),
            Value::Sequence(SequenceValue::BoundedStringSequence(v)) => {
                let size = self.current.base_type.size().unwrap_or(1);
                let start = idx * size + offset;
                let bytes = &self.storage[start..start + size];
                unsafe { Ok(Value::Simple(SimpleValue::BoundedString(DynamicBoundedString::new(bytes, v[idx].upper_bound())))) }
            },
            Value::Sequence(SequenceValue::BoundedWStringSequence(v)) => {
                let size = self.current.base_type.size().unwrap_or(1);
                let start = idx * size + offset;
                let bytes = &self.storage[start..start + size];
                unsafe { Ok(Value::Simple(SimpleValue::BoundedWString(DynamicBoundedWString::new(bytes, v[idx].upper_bound())))) }
            },
            Value::BoundedSequence(BoundedSequenceValue::BooleanBoundedSequence(v)) => {
                let b = v.get(idx).ok_or(super::DynamicMessageError::InvalidEncapsulation)?;
                // SAFETY: The pointer ultimately points to the underlying storage, which lives long enough
                Ok(Value::Simple(SimpleValue::Boolean(unsafe { (b as *const bool).as_ref().expect("Valid ptr") })))
            },
            Value::BoundedSequence(BoundedSequenceValue::Int8BoundedSequence(v)) => {
                let i = v.get(idx).ok_or(super::DynamicMessageError::InvalidEncapsulation)?;
                // SAFETY: The pointer ultimately points to the underlying storage, which lives long enough
                Ok(Value::Simple(SimpleValue::Int8(unsafe { (i as *const i8).as_ref().expect("Valid ptr") })))
            },
            Value::BoundedSequence(BoundedSequenceValue::Uint8BoundedSequence(v)) => {
                let u = v.get(idx).ok_or(super::DynamicMessageError::InvalidEncapsulation)?;
                // SAFETY: The pointer ultimately points to the underlying storage, which lives long enough
                Ok(Value::Simple(SimpleValue::Uint8(unsafe { (u as *const u8).as_ref().expect("Valid ptr") })))
            },
            Value::BoundedSequence(BoundedSequenceValue::Int16BoundedSequence(v)) => {
                let i = v.get(idx).ok_or(super::DynamicMessageError::InvalidEncapsulation)?;
                // SAFETY: The pointer ultimately points to the underlying storage, which lives long enough
                Ok(Value::Simple(SimpleValue::Int16(unsafe { (i as *const i16).as_ref().expect("Valid ptr") })))
            },
            Value::BoundedSequence(BoundedSequenceValue::Uint16BoundedSequence(v)) => {
                let u = v.get(idx).ok_or(super::DynamicMessageError::InvalidEncapsulation)?;
                // SAFETY: The pointer ultimately points to the underlying storage, which lives long enough
                Ok(Value::Simple(SimpleValue::Uint16(unsafe { (u as *const u16).as_ref().expect("Valid ptr") })))
            },
            Value::BoundedSequence(BoundedSequenceValue::Int32BoundedSequence(v)) => {
                let i = v.get(idx).ok_or(super::DynamicMessageError::InvalidEncapsulation)?;
                // SAFETY: The pointer ultimately points to the underlying storage, which lives long enough
                Ok(Value::Simple(SimpleValue::Int32(unsafe { (i as *const i32).as_ref().expect("Valid ptr") })))
            },
            Value::BoundedSequence(BoundedSequenceValue::Uint32BoundedSequence(v)) => {
                let u = v.get(idx).ok_or(super::DynamicMessageError::InvalidEncapsulation)?;
                // SAFETY: The pointer ultimately points to the underlying storage, which lives long enough
                Ok(Value::Simple(SimpleValue::Uint32(unsafe { (u as *const u32).as_ref().expect("Valid ptr") })))
            },
            Value::BoundedSequence(BoundedSequenceValue::Int64BoundedSequence(v)) => {
                let i = v.get(idx).ok_or(super::DynamicMessageError::InvalidEncapsulation)?;
                // SAFETY: The pointer ultimately points to the underlying storage, which lives long enough
                Ok(Value::Simple(SimpleValue::Int64(unsafe { (i as *const i64).as_ref().expect("Valid ptr") })))
            },
            Value::BoundedSequence(BoundedSequenceValue::Uint64BoundedSequence(v)) => {
                let u = v.get(idx).ok_or(super::DynamicMessageError::InvalidEncapsulation)?;
                // SAFETY: The pointer ultimately points to the underlying storage, which lives long enough
                Ok(Value::Simple(SimpleValue::Uint64(unsafe { (u as *const u64).as_ref().expect("Valid ptr") })))
            },
            Value::BoundedSequence(BoundedSequenceValue::FloatBoundedSequence(v)) => {
                let f = v.get(idx).ok_or(super::DynamicMessageError::InvalidEncapsulation)?;
                // SAFETY: The pointer ultimately points to the underlying storage, which lives long enough
                Ok(Value::Simple(SimpleValue::Float(unsafe { (f as *const f32).as_ref().expect("Valid ptr") })))
            },
            Value::BoundedSequence(BoundedSequenceValue::DoubleBoundedSequence(v)) => {
                let d = v.get(idx).ok_or(super::DynamicMessageError::InvalidEncapsulation)?;
                // SAFETY: The pointer ultimately points to the underlying storage, which lives long enough
                Ok(Value::Simple(SimpleValue::Double(unsafe { (d as *const f64).as_ref().expect("Valid ptr") })))
            },
            Value::BoundedSequence(BoundedSequenceValue::StringBoundedSequence(v)) => {
                let s = v.get(idx).ok_or(super::DynamicMessageError::InvalidEncapsulation)?;
                Ok(Value::Simple(SimpleValue::String(unsafe { (s as *const rosidl_runtime_rs::String).as_ref().expect("Valid ptr") })))
            },
            Value::BoundedSequence(BoundedSequenceValue::WStringBoundedSequence(v)) => {
                let s = v.get(idx).ok_or(super::DynamicMessageError::InvalidEncapsulation)?;
                Ok(Value::Simple(SimpleValue::WString(unsafe { (s as *const rosidl_runtime_rs::WString).as_ref().expect("Valid ptr") })))
            },
            Value::BoundedSequence(BoundedSequenceValue::CharBoundedSequence(v)) => {
                let c = v.get(idx).ok_or(super::DynamicMessageError::InvalidEncapsulation)?;
                Ok(Value::Simple(SimpleValue::Char(unsafe { (c as *const u8).as_ref().expect("Valid ptr") })))
            },
            Value::BoundedSequence(BoundedSequenceValue::WCharBoundedSequence(v)) => {
                let c = v.get(idx).ok_or(super::DynamicMessageError::InvalidEncapsulation)?;
                Ok(Value::Simple(SimpleValue::WChar(unsafe { (c as *const u16).as_ref().expect("Valid ptr") })))
            },
            Value::BoundedSequence(BoundedSequenceValue::OctetBoundedSequence(v)) => {
                let o = v.get(idx).ok_or(super::DynamicMessageError::InvalidEncapsulation)?;
                Ok(Value::Simple(SimpleValue::Octet(unsafe { (o as *const u8).as_ref().expect("Valid ptr") })))
            },
            Value::BoundedSequence(BoundedSequenceValue::BoundedStringBoundedSequence(v)) => {
                let size = self.current.base_type.size().unwrap_or(1);
                let start = idx * size + offset;
                let bytes = &self.storage[start..start + size];
                unsafe { Ok(Value::Simple(SimpleValue::BoundedString(DynamicBoundedString::new(bytes, v[idx].upper_bound())))) }
            },
            Value::BoundedSequence(BoundedSequenceValue::BoundedWStringBoundedSequence(v)) => {
                let size = self.current.base_type.size().unwrap_or(1);
                let start = idx * size + offset;
                let bytes = &self.storage[start..start + size];
                unsafe { Ok(Value::Simple(SimpleValue::BoundedWString(DynamicBoundedWString::new(bytes, v[idx].upper_bound())))) }
            },
            ty => Err(super::DynamicMessageError::TypeNotSupported(format!("{:?}", ty))),
        }
    }

    fn set_current(&mut self) -> Result<()> {
        if self.pos.is_empty() {
            return Ok(());
        }
        let mut current = self.structure.fields.get(self.pos.first().unwrap_or(&(0, 0, false)).0).ok_or(
            super::DynamicMessageError::InvalidEncapsulation,
        )?;
        let mut offset: usize = 0;
        let mut skip_seq = false;
        for (idx, _, _) in self.pos.iter().skip(1) {
            if current.value_kind == ValueKind::Simple || skip_seq {
                if let BaseType::Message(ref msg_struct) = current.base_type {
                    offset += current.offset;
                    current = msg_struct.fields.get(*idx).ok_or(
                        super::DynamicMessageError::InvalidEncapsulation,
                    )?;
                }
                skip_seq = false;
            } else {
                // We are in a sequence, so skip the current idx and treat the next as a field index
                skip_seq = true;
            }
        }
        self.current = current;
        self.offset = offset;
        Ok(())
    }

    fn array_len(&self) -> Result<usize> {
        let size = self.current.size().unwrap_or(1);
        let offset = self.offset + self.current.offset;
        // SAFETY: The byte slice contains a valid field as assured by DynamicMessage construction
        let value = unsafe { Value::new(&self.storage[offset..offset + size], self.current) };
        let value = value.expect("Valid value");
        let len = match value {
            Value::Array(_) => match self.current.value_kind {
                ValueKind::Array { length } => length,
                _ => {
                    return Err(super::DynamicMessageError::TypeNotSupported(
                        "Expected array value".to_string(),
                    ))
                }
            }
            Value::Sequence(SequenceValue::BooleanSequence(seq)) => seq.len(),
            Value::Sequence(SequenceValue::Int8Sequence(seq)) => seq.len(),
            Value::Sequence(SequenceValue::Uint8Sequence(seq)) => seq.len(),
            Value::Sequence(SequenceValue::Int16Sequence(seq)) => seq.len(),
            Value::Sequence(SequenceValue::Uint16Sequence(seq)) => seq.len(),
            Value::Sequence(SequenceValue::Int32Sequence(seq)) => seq.len(),
            Value::Sequence(SequenceValue::Uint32Sequence(seq)) => seq.len(),
            Value::Sequence(SequenceValue::Int64Sequence(seq)) => seq.len(),
            Value::Sequence(SequenceValue::Uint64Sequence(seq)) => seq.len(),
            Value::Sequence(SequenceValue::FloatSequence(seq)) => seq.len(),
            Value::Sequence(SequenceValue::DoubleSequence(seq)) => seq.len(),
            Value::Sequence(SequenceValue::StringSequence(seq)) => seq.len(),
            Value::Sequence(SequenceValue::WStringSequence(seq)) => seq.len(),
            Value::Sequence(SequenceValue::CharSequence(seq)) => seq.len(),
            Value::Sequence(SequenceValue::OctetSequence(seq)) => seq.len(),
            Value::Sequence(SequenceValue::BoundedStringSequence(seq)) => seq.len(),
            Value::Sequence(SequenceValue::BoundedWStringSequence(seq)) => seq.len(),
            Value::Sequence(SequenceValue::MessageSequence(seq)) => seq.len(),
            Value::BoundedSequence(BoundedSequenceValue::BooleanBoundedSequence(seq)) => seq.len(),
            Value::BoundedSequence(BoundedSequenceValue::Int8BoundedSequence(seq)) => seq.len(),
            Value::BoundedSequence(BoundedSequenceValue::Uint8BoundedSequence(seq)) => seq.len(),
            Value::BoundedSequence(BoundedSequenceValue::Int16BoundedSequence(seq)) => seq.len(),
            Value::BoundedSequence(BoundedSequenceValue::Uint16BoundedSequence(seq)) => seq.len(),
            Value::BoundedSequence(BoundedSequenceValue::Int32BoundedSequence(seq)) => seq.len(),
            Value::BoundedSequence(BoundedSequenceValue::Uint32BoundedSequence(seq)) => seq.len(),
            Value::BoundedSequence(BoundedSequenceValue::Int64BoundedSequence(seq)) => seq.len(),
            Value::BoundedSequence(BoundedSequenceValue::Uint64BoundedSequence(seq)) => seq.len(),
            Value::BoundedSequence(BoundedSequenceValue::FloatBoundedSequence(seq)) => seq.len(),
            Value::BoundedSequence(BoundedSequenceValue::DoubleBoundedSequence(seq)) => seq.len(),
            Value::BoundedSequence(BoundedSequenceValue::StringBoundedSequence(seq)) => seq.len(),
            Value::BoundedSequence(BoundedSequenceValue::WStringBoundedSequence(seq)) => seq.len(),
            Value::BoundedSequence(BoundedSequenceValue::CharBoundedSequence(seq)) => seq.len(),
            Value::BoundedSequence(BoundedSequenceValue::OctetBoundedSequence(seq)) => seq.len(),
            Value::BoundedSequence(BoundedSequenceValue::BoundedStringBoundedSequence(seq)) => seq.len(),
            Value::BoundedSequence(BoundedSequenceValue::BoundedWStringBoundedSequence(seq)) => seq.len(),
            Value::BoundedSequence(BoundedSequenceValue::MessageBoundedSequence(seq)) => seq.len(),
            _ => {
                return Err(super::DynamicMessageError::TypeNotSupported(
                    "array_len called on non-seq type".to_string(),
                ))
            }
        };
        Ok(len)
    }

    fn inc_seq(&mut self, push: bool) -> Result<bool> {
        if push {
            if self.current.value_kind == ValueKind::Simple {
                return Err(super::DynamicMessageError::TypeNotSupported(
                    "Cannot increment sequence on non-sequence type".to_string(),
                ));
            }
            let len = self.array_len()?;
            if len == 0 {
                return Ok(false);
            }
            self.pos.push((0, len, true));
            return Ok(true);
        }
        let pos_len = *self.pos.last().expect("pos stack should not be empty");
        // Sanity check to make sure we are not out of bounds
        if pos_len.0 >= pos_len.1 {
            return Err(super::DynamicMessageError::InvalidEncapsulation);
        }
        // If we have reached the end of the sequence, pop and return false
        if pos_len.0 == pos_len.1 - 1 {
            self.pos.pop();
            return Ok(false);
        }
        // Otherwise, increment the position and return true
        let pos = self.pos.last_mut().expect("pos stack should not be empty");
        pos.0 += 1;
        Ok(true)
    }

    fn inc_field(&mut self, push: bool) -> Result<bool> {
        if push {
            // Top-level (just started deserialization)
            if self.pos.is_empty() {
                self.pos.push((0, self.structure.fields.len(), false));
                self.set_current()?;
                return Ok(true);
            }
            // We SHOULD have an inner message type to start parsing
            let len = match &self.current.base_type {
                BaseType::Message(msg) => msg.fields.len(),
                _ => {
                    return Err(super::DynamicMessageError::TypeNotSupported(
                        "Cannot push field on non-message type".to_string(),
                    ))
                }
            };
            // Not sure if a zero-field message is possible, but just in case
            if len == 0 {
                return Ok(false);
            }
            self.pos.push((0, len, false));
            self.set_current()?;
            return Ok(true);
        }
        let mut pos_len = *self.pos.last().expect("pos stack should not be empty");
        // The last pos may still be in a sequence, so we need to pop back
        if pos_len.2 {
            self.pos.pop();
            pos_len = *self.pos.last().expect("pos stack should not be empty");
        }
        // Sanity check to make sure we are not out of bounds
        if pos_len.0 >= pos_len.1 {
            return Err(super::DynamicMessageError::InvalidEncapsulation);
        }
        // If we have reached the end of the fields, pop and return false
        if pos_len.0 == pos_len.1 - 1 {
            self.pos.pop();
            self.set_current()?;
            return Ok(false);
        }
        // Otherwise, increment the position and return true
        let pos = self.pos.last_mut().expect("pos stack should not be empty");
        pos.0 += 1;
        self.set_current()?;
        Ok(true)
    }
}

impl<'de> de::Deserializer<'de> for &'_ mut DynReader<'de> {
    type Error = super::DynamicMessageError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let pos = *self.pos.last().unwrap_or(&(0, 0, false));
        let mut value_kind = &self.current.value_kind;
        // If we are in an array/seq, deserialize the inner type
        if pos.2 {
            value_kind = &ValueKind::Simple;
        }
        match value_kind {
            ValueKind::Simple => match &self.current.base_type {
                BaseType::Boolean => self.deserialize_bool(visitor),
                BaseType::Octet => self.deserialize_u8(visitor),
                BaseType::Char => self.deserialize_char(visitor),
                BaseType::Float => self.deserialize_f32(visitor),
                BaseType::Double => self.deserialize_f64(visitor),
                BaseType::Int8 => self.deserialize_i8(visitor),
                BaseType::Uint8 => self.deserialize_u8(visitor),
                BaseType::Int16 => self.deserialize_i16(visitor),
                BaseType::Uint16 => self.deserialize_u16(visitor),
                BaseType::Int32 => self.deserialize_i32(visitor),
                BaseType::Uint32 => self.deserialize_u32(visitor),
                BaseType::Int64 => self.deserialize_i64(visitor),
                BaseType::Uint64 => self.deserialize_u64(visitor),
                BaseType::String => self.deserialize_str(visitor),
                BaseType::BoundedString { upper_bound: _ } => self.deserialize_str(visitor),
                BaseType::WChar => self.deserialize_char(visitor),
                BaseType::WString => self.deserialize_str(visitor),
                BaseType::BoundedWString { upper_bound: _ } => self.deserialize_str(visitor),
                BaseType::Message(_msg) => self.deserialize_map(visitor),
                ty => Err(Self::Error::TypeNotSupported(format!("{:?}", ty))),
            },
            ValueKind::Array { length } => self.deserialize_tuple(*length, visitor),
            ValueKind::Sequence => self.deserialize_seq(visitor),
            ValueKind::BoundedSequence { upper_bound: _ } => self.deserialize_seq(visitor),
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        match self.get_value()? {
            Value::Simple(SimpleValue::Boolean(b)) => {
                visitor.visit_bool(*b)
            }
            ty => Err(Self::Error::TypeNotSupported(format!("{:?}", ty))),
        }
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        match self.get_value()? {
            Value::Array(ArrayValue::CharArray(b)) => visitor.visit_bytes(b),
            Value::Array(ArrayValue::OctetArray(b)) => visitor.visit_bytes(b),
            ty => Err(Self::Error::TypeNotSupported(format!("{:?}", ty))),
        }
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        match self.get_value()? {
            Value::Array(ArrayValue::CharArray(b)) => visitor.visit_byte_buf(b.to_vec()),
            Value::Array(ArrayValue::OctetArray(b)) => visitor.visit_byte_buf(b.to_vec()),
            ty => Err(Self::Error::TypeNotSupported(format!("{:?}", ty))),
        }
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        match self.get_value()? {
            Value::Simple(SimpleValue::Int8(i)) => visitor.visit_i8(*i),
            ty => Err(Self::Error::TypeNotSupported(format!("{:?}", ty))),
        }
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        match self.get_value()? {
            Value::Simple(SimpleValue::Int16(i)) => visitor.visit_i16(*i),
            ty => Err(Self::Error::TypeNotSupported(format!("{:?}", ty))),
        }
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        match self.get_value()? {
            Value::Simple(SimpleValue::Int32(i)) => visitor.visit_i32(*i),
            ty => Err(Self::Error::TypeNotSupported(format!("{:?}", ty))),
        }
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        match self.get_value()? {
            Value::Simple(SimpleValue::Int64(i)) => visitor.visit_i64(*i),
            ty => Err(Self::Error::TypeNotSupported(format!("{:?}", ty))),
        }
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        match self.get_value()? {
            Value::Simple(SimpleValue::Uint8(u)) => visitor.visit_u8(*u),
            Value::Simple(SimpleValue::Char(c)) => visitor.visit_u8(*c),
            Value::Simple(SimpleValue::Octet(o)) => visitor.visit_u8(*o),
            ty => Err(Self::Error::TypeNotSupported(format!("{:?}", ty))),
        }
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        match self.get_value()? {
            Value::Simple(SimpleValue::Uint16(u)) => visitor.visit_u16(*u),
            ty => Err(Self::Error::TypeNotSupported(format!("{:?}", ty))),
        }
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        match self.get_value()? {
            Value::Simple(SimpleValue::Uint32(u)) => visitor.visit_u32(*u),
            ty => Err(Self::Error::TypeNotSupported(format!("{:?}", ty))),
        }
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        match self.get_value()? {
            Value::Simple(SimpleValue::Uint64(u)) => visitor.visit_u64(*u),
            ty => Err(Self::Error::TypeNotSupported(format!("{:?}", ty))),
        }
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        match self.get_value()? {
            Value::Simple(SimpleValue::Float(f)) => visitor.visit_f32(*f),
            ty => Err(Self::Error::TypeNotSupported(format!("{:?}", ty))),
        }
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        match self.get_value()? {
            Value::Simple(SimpleValue::Double(f)) => visitor.visit_f64(*f),
            ty => Err(Self::Error::TypeNotSupported(format!("{:?}", ty))),
        }
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        match self.get_value()? {
            Value::Simple(SimpleValue::Char(c)) => visitor.visit_char((*c).into()),
            Value::Simple(SimpleValue::Uint8(u)) => visitor.visit_char((*u).into()),
            Value::Simple(SimpleValue::Octet(o)) => visitor.visit_char((*o).into()),
            ty => Err(Self::Error::TypeNotSupported(format!("{:?}", ty))),
        }
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        match self.get_value()? {
            Value::Simple(SimpleValue::String(s)) => {
                visitor.visit_str(s.to_cstr().to_str().map_err(|_| Self::Error::InvalidUtf8Encoding)?)
            }
            Value::Simple(SimpleValue::BoundedString(s)) => {
                visitor.visit_str(s.to_cstr().to_str().map_err(|_| Self::Error::InvalidUtf8Encoding)?)
            }
            Value::Simple(SimpleValue::WString(s)) => {
                visitor.visit_string(String::from_utf16(s).map_err(|_| Self::Error::InvalidUtf8Encoding)?)
            }
            Value::Simple(SimpleValue::BoundedWString(s)) => {
                visitor.visit_string(String::from_utf16(&s).map_err(|_| Self::Error::InvalidUtf8Encoding)?)
            }
            ty => Err(Self::Error::TypeNotSupported(format!("{:?}", ty))),
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        match self.get_value()? {
            Value::Simple(SimpleValue::String(s)) => {
                visitor.visit_string(s.to_cstr().to_str().map_err(|_| Self::Error::InvalidUtf8Encoding)?.to_owned())
            }
            Value::Simple(SimpleValue::BoundedString(s)) => {
                visitor.visit_string(s.to_cstr().to_str().map_err(|_| Self::Error::InvalidUtf8Encoding)?.to_owned())
            }
            Value::Simple(SimpleValue::WString(s)) => {
                visitor.visit_string(String::from_utf16(s).map_err(|_| Self::Error::InvalidUtf8Encoding)?)
            }
            Value::Simple(SimpleValue::BoundedWString(s)) => {
                visitor.visit_string(String::from_utf16(&s).map_err(|_| Self::Error::InvalidUtf8Encoding)?)
            }
            ty => Err(Self::Error::TypeNotSupported(format!("{:?}", ty))),
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_option<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        Err(Self::Error::TypeNotSupported("rosidl does not support Option<T>".to_string()))
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_seq(SeqAccess {
            de: self,
            len: None,
            first: true,
        })
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_seq(SeqAccess {
            de: self,
            len: Some(len),
            first: true,
        })
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_seq(SeqAccess {
            de: self,
            len: Some(len),
            first: true,
        })
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> std::result::Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_enum(self)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_map(MsgAccess::new(self))
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_map(MsgAccess::new(self))
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_str(self.current.name.as_str())
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    fn is_human_readable(&self) -> bool {
        false
    }
}

impl<'de> de::VariantAccess<'de> for &'_ mut DynReader<'de> {
    type Error = super::DynamicMessageError;

    fn unit_variant(self) -> Result<()> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: de::DeserializeSeed<'de>,
    {
        de::DeserializeSeed::deserialize(seed, self)
    }

    fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        de::Deserializer::deserialize_tuple(self, len, visitor)
    }

    fn struct_variant<V>(self, fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        de::Deserializer::deserialize_struct(self, "", fields, visitor)
    }
}

impl<'de> de::EnumAccess<'de> for &'_ mut DynReader<'de> {
    type Error = super::DynamicMessageError;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: de::DeserializeSeed<'de>,
    {
        let variant = seed.deserialize(&mut *self)?;
        Ok((variant, self))
    }
}

struct SeqAccess<'a, 'de> {
    de: &'a mut DynReader<'de>,
    len: Option<usize>,
    first: bool,
}

impl<'de, 'a> de::SeqAccess<'de> for SeqAccess<'a, 'de> {
    type Error = super::DynamicMessageError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: de::DeserializeSeed<'de>,
    {
        if self.de.inc_seq(self.first)? {
            self.first = false;
            seed.deserialize(&mut *self.de).map(Some)
        } else {
            Ok(None)
        }
    }

    fn size_hint(&self) -> Option<usize> {
        self.len
    }
}

struct MsgAccess<'a, 'de> {
    de: &'a mut DynReader<'de>,
    first: bool,
}

impl<'a, 'de> MsgAccess<'a, 'de> {
    fn new(de: &'a mut DynReader<'de>) -> Self {
        Self { de, first: true }
    }
}

impl<'de, 'a> de::MapAccess<'de> for MsgAccess<'a, 'de> {
    type Error = super::DynamicMessageError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: de::DeserializeSeed<'de>,
    {
        if self.de.inc_field(self.first)? {
            self.first = false;
            seed.deserialize(&mut *self.de).map(Some)
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: de::DeserializeSeed<'de>,
    {
        seed.deserialize(&mut *self.de)
    }
}

/// Deserialize a Rust type from a `DynamicMessageView`.
pub fn from_dyn_msg_view<'a, T>(msg: DynamicMessageView<'a>) -> Result<T>
where
    T: Deserialize<'a>,
{
    let mut deserializer = DynReader::from_dyn_msg_view(msg);
    let t = T::deserialize(&mut deserializer)?;
    Ok(t)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DynamicMessage, vendor::test_msgs::msg::{Constants, Defaults, MultiNested}};
    use serde::{Deserialize, Serialize};
    use crate::vendor::test_msgs::msg::{BasicTypes, BoundedSequences, Strings, Arrays, UnboundedSequences};

    // Basic type tests
    #[test]
    fn test_deserialize_basic_types() {
        let msg = DynamicMessage::new("test_msgs/msg/BasicTypes".try_into().unwrap()).unwrap();
        let view = msg.view();

        let result = from_dyn_msg_view::<BasicTypes>(view);

        assert!(result.is_ok());
    }

    #[test]
    fn test_deserialize_arrays() {
        let msg = DynamicMessage::new("test_msgs/msg/Arrays".try_into().unwrap()).unwrap();
        let view = msg.view();

        let result = from_dyn_msg_view::<Arrays>(view);
        assert!(result.is_ok());
    }

    #[test]
    fn test_deserialize_strings() {
        let msg = DynamicMessage::new("test_msgs/msg/Strings".try_into().unwrap()).unwrap();
        let view = msg.view();

        let result = from_dyn_msg_view::<Strings>(view);
        assert!(result.is_ok());
    }

    #[test]
    fn test_deserialize_bounded_sequences() {
        let msg =
            DynamicMessage::new("test_msgs/msg/BoundedSequences".try_into().unwrap()).unwrap();
        let view = msg.view();

        let result = from_dyn_msg_view::<BoundedSequences>(view);
        assert!(result.is_ok());
    }

    #[test]
    fn test_deserialize_unbounded_sequences() {
        let msg =
            DynamicMessage::new("test_msgs/msg/UnboundedSequences".try_into().unwrap()).unwrap();
        let view = msg.view();

        let result = from_dyn_msg_view::<UnboundedSequences>(view);
        assert!(result.is_ok());
    }

    #[test]
    fn test_deserialize_empty_message() {
        let msg = DynamicMessage::new("test_msgs/msg/Empty".try_into().unwrap()).unwrap();
        let view = msg.view();

        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        struct EmptyMessage {
            structure_needs_at_least_one_member: u8,
        }

        let result = from_dyn_msg_view::<EmptyMessage>(view);
        assert!(result.is_ok());
    }

    #[test]
    fn test_deserialize_nested_message() {
        let msg = DynamicMessage::new("test_msgs/msg/Nested".try_into().unwrap()).unwrap();
        let view = msg.view();

        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        struct NestedMessage {
            basic_types_value: BasicTypes,
        }

        let result = from_dyn_msg_view::<NestedMessage>(view);
        assert!(result.is_ok());
    }

    #[test]
    fn test_deserialize_multi_nested_message() {
        let msg = DynamicMessage::new("test_msgs/msg/MultiNested".try_into().unwrap()).unwrap();
        let view = msg.view();

        let result = from_dyn_msg_view::<MultiNested>(view);
        assert!(result.is_ok());
    }

    #[test]
    fn test_deserialize_with_defaults() {
        let msg = DynamicMessage::new("test_msgs/msg/Defaults".try_into().unwrap()).unwrap();
        let view = msg.view();

        let result = from_dyn_msg_view::<Defaults>(view);
        assert!(result.is_ok());
    }

    #[test]
    fn test_deserialize_constants() {
        let msg = DynamicMessage::new("test_msgs/msg/Constants".try_into().unwrap()).unwrap();
        let view = msg.view();

        let result = from_dyn_msg_view::<Constants>(view);
        assert!(result.is_ok());
    }

    // Test error handling
    #[test]
    fn test_field_not_found_error() {
        let msg = DynamicMessage::new("test_msgs/msg/BasicTypes".try_into().unwrap()).unwrap();
        let view = msg.view();

        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        struct WrongFieldName {
            nonexistent_field: i32,
        }

        let result = from_dyn_msg_view::<WrongFieldName>(view);
        assert!(result.is_err());
    }

    #[test]
    fn test_type_not_supported_error() {
        let msg = DynamicMessage::new("test_msgs/msg/BasicTypes".try_into().unwrap()).unwrap();
        let view = msg.view();

        // Try to deserialize as wrong type to trigger type error
        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        struct WrongType {
            bool_value: String, // bool field as string should fail
        }

        let result = from_dyn_msg_view::<WrongType>(view);
        assert!(result.is_err());
    }

    #[test]
    fn test_deserialize_option_not_supported() {
        let msg = DynamicMessage::new("test_msgs/msg/BasicTypes".try_into().unwrap()).unwrap();
        let view = msg.view();

        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        struct WithOption {
            int32_value: Option<i32>,
        }

        let result = from_dyn_msg_view::<WithOption>(view);
        // Should fail since Option is not supported
        assert!(result.is_err());
    }

    #[test]
    fn test_deserialize_map_not_supported() {
        let msg = DynamicMessage::new("test_msgs/msg/BasicTypes".try_into().unwrap()).unwrap();
        let view = msg.view();

        use std::collections::HashMap;
        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        struct WithMap {
            map_field: HashMap<String, i32>,
        }

        let result = from_dyn_msg_view::<WithMap>(view);
        // Should fail since Map is not supported
        assert!(result.is_err());
    }

    // Test tuple and tuple struct deserialization
    #[test]
    fn test_deserialize_tuple() {
        let msg = DynamicMessage::new("test_msgs/msg/Arrays".try_into().unwrap()).unwrap();
        let view = msg.view();

        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        struct TupleStruct {
            int32_values: (i32, i32, i32),
        }

        let result = from_dyn_msg_view::<TupleStruct>(view);
        assert!(result.is_ok());
    }

    #[test]
    fn test_deserialize_tuple_struct() {
        let msg = DynamicMessage::new("test_msgs/msg/Arrays".try_into().unwrap()).unwrap();
        let view = msg.view();

        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        struct TupleStruct(bool, i32, f32);

        let result = from_dyn_msg_view::<TupleStruct>(view);
        assert!(result.is_err());
    }

    // Test newtype deserialization
    #[test]
    fn test_deserialize_newtype() {
        let msg = DynamicMessage::new("test_msgs/msg/BasicTypes".try_into().unwrap()).unwrap();
        let view = msg.view();

        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        struct NewTypeWrapper(i32);

        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        struct WithNewType {
            #[serde(rename = "int32_value")]
            wrapped: NewTypeWrapper,
        }

        let result = from_dyn_msg_view::<WithNewType>(view);
        assert!(result.is_ok());
    }

    // Test unit and unit struct deserialization
    #[test]
    fn test_deserialize_unit() {
        let msg = DynamicMessage::new("test_msgs/msg/BasicTypes".try_into().unwrap()).unwrap();
        let view = msg.view();

        let result = from_dyn_msg_view::<()>(view);
        assert!(result.is_ok());
    }

    #[test]
    fn test_deserialize_unit_struct() {
        let msg = DynamicMessage::new("test_msgs/msg/BasicTypes".try_into().unwrap()).unwrap();
        let view = msg.view();

        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        struct UnitStruct;

        let result = from_dyn_msg_view::<UnitStruct>(view);
        assert!(result.is_ok());
    }

    // Test byte arrays and sequences
    #[test]
    fn test_deserialize_bytes() {
        let mut msg = DynamicMessage::new("test_msgs/msg/UnboundedSequences".try_into().unwrap()).unwrap();

        if let Some(crate::ValueMut::Sequence(crate::SequenceValueMut::Int8Sequence(seq))) =
            msg.get_mut("int8_values")
        {
            // Populate with 64 int8 values
            seq.extend(0..64);
        }

        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        struct BytesStruct {
            #[serde(with = "serde_big_array::BigArray")]
            int8_values: [i8; 64],
        }

        let view = msg.view();
        let result = from_dyn_msg_view::<BytesStruct>(view);
        assert!(result.is_ok());
    }

    // Integration tests with realistic data
    #[test]
    fn test_deserialize_with_populated_data() {
        let mut msg = DynamicMessage::new("test_msgs/msg/BasicTypes".try_into().unwrap()).unwrap();

        // Set some values in the message
        if let Some(crate::ValueMut::Simple(crate::SimpleValueMut::Boolean(b))) =
            msg.get_mut("bool_value")
        {
            *b = true;
        }

        if let Some(crate::ValueMut::Simple(crate::SimpleValueMut::Int32(i))) =
            msg.get_mut("int32_value")
        {
            *i = 42;
        }

        if let Some(crate::ValueMut::Simple(crate::SimpleValueMut::Float(f))) =
            msg.get_mut("float32_value")
        {
            *f = std::f32::consts::PI;
        }

        let view = msg.view();
        let result = from_dyn_msg_view::<BasicTypes>(view);

        assert!(result.is_ok());
    }

    #[test]
    fn test_deserialize_all_primitive_types() {
        let msg = DynamicMessage::new("test_msgs/msg/BasicTypes".try_into().unwrap()).unwrap();

        // Test deserialization of each primitive type individually
        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        struct BoolOnly {
            bool_value: bool,
        }

        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        struct Int32Only {
            int32_value: i32,
        }

        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        struct ByteOnly {
            byte_value: u8,
        }

        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        struct FloatOnly {
            float32_value: f32,
        }

        // Test each type separately
        let bool_result = from_dyn_msg_view::<BoolOnly>(msg.view());
        let int_result = from_dyn_msg_view::<Int32Only>(msg.view());
        let byte_result = from_dyn_msg_view::<ByteOnly>(msg.view());
        let float_result = from_dyn_msg_view::<FloatOnly>(msg.view());

        assert!(bool_result.is_ok());
        assert!(int_result.is_ok());
        assert!(byte_result.is_ok());
        assert!(float_result.is_ok());
    }

    // Test with empty struct (edge case)
    #[test]
    fn test_deserialize_empty_struct() {
        let msg = DynamicMessage::new("test_msgs/msg/Empty".try_into().unwrap()).unwrap();
        let view = msg.view();

        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        struct Empty {}

        let result = from_dyn_msg_view::<Empty>(view);
        assert!(result.is_ok());
    }

    #[test]
    fn test_deserialize_large_sequence() {
        let mut msg =
            DynamicMessage::new("test_msgs/msg/UnboundedSequences".try_into().unwrap()).unwrap();

        if let Some(crate::ValueMut::Sequence(crate::SequenceValueMut::Int32Sequence(seq))) =
            msg.get_mut("int32_values")
        {
            // Populate with a large number of elements
            seq.extend(0..1000);
        }

        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        struct LargeSequenceTest {
            int32_values: Vec<i32>,
        }

        let view = msg.view();
        let result = from_dyn_msg_view::<LargeSequenceTest>(view);
        assert!(result.is_ok());
    }
}
