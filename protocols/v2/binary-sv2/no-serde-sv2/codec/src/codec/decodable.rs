//! This module provides an interface and implementation details for decoding complex data structures
//! from raw bytes or I/O streams. It is intended to handle deserialization of nested and primitive data
//! structures by defining traits, enums, and helper functions for managing the decoding process.
//!
//! # Overview
//! The core component of this module is the `Decodable` trait, which provides methods for defining the
//! structure of a type, managing the decoding of raw byte data, and constructing instances of the type
//! from decoded fields. This trait is designed to work with both in-memory byte slices and I/O streams,
//! making it flexible for various deserialization use cases.
//!
//! # Key Concepts and Types
//! - **`Decodable` Trait**: The primary trait for decoding types from byte data. It includes methods to
//!   break down raw data, decode individual fields, and construct the final type.
//! - **`FieldMarker` and `PrimitiveMarker`**: These enums represent different data types or structures.
//!   They guide the decoding process by defining the structure of fields and their respective types.
//! - **`DecodableField` and `DecodablePrimitive`**: Variants for decoded fields, representing either
//!   primitive types or nested structures. They form the building blocks for assembling complex data types.
//! - **`SizeHint`**: Provides size information for fields and structures to assist in efficient decoding.
//!
//! # Error Handling
//! This module defines custom error types to handle issues that may arise during decoding,
//! such as insufficient data or unsupported types. Errors are surfaced through `Result`
//! types and managed gracefully to ensure reliability in data parsing tasks.
//!
//! # `no_std` Support
//! The module is compatible with `no_std` environments by conditional compilation. When
//! the `no_std` feature is enabled, I/O-dependent methods like `from_reader` are omitted,
//! allowing for a lightweight build in constrained environments.
use crate::{
    codec::{GetSize, SizeHint},
    datatypes::{
        ShortTxId, Signature, Sv2DataType, U32AsRef, B016M, B0255, B032, B064K, U24, U256,
    },
    Error,
};
use alloc::vec::Vec;
use core::convert::TryFrom;
#[cfg(not(feature = "no_std"))]
use std::io::{Cursor, Read};

/// Trait that defines how a type can be decoded from raw byte data.
///
/// This trait describes the process of decoding a data structure from a sequence of bytes.
/// Implementations use a combination of methods to extract the structure of the data, decode its
/// fields, and then construct the type from those decoded fields. It is designed to handle both
/// simple types and nested or complex data structures.
///
/// - `get_structure`: Describes the layout of the type's fields, allowing the decoder to break down the raw data.
/// - `from_decoded_fields`: Reconstructs the type from individual decoded fields.
/// - `from_bytes`: High-level method that manages the decoding process from raw bytes.
/// - `from_reader`: Reads and decodes data from a stream, useful when working with I/O sources like files or network sockets.
pub trait Decodable<'a>: Sized {
    // Returns the structure of the type.
    //
    // This method defines the layout of the data fields within the type. The structure
    // returned is used to split raw data into individual fields that can be decoded.
    fn get_structure(data: &[u8]) -> Result<Vec<FieldMarker>, Error>;

    // Constructs the type from decoded fields.
    //
    // After the data has been split into fields, this method combines those fields
    // back into the original type, handling nested structures or composite fields.
    fn from_decoded_fields(data: Vec<DecodableField<'a>>) -> Result<Self, Error>;

    // Decodes the type from raw bytes.
    //
    // This method orchestrates the decoding process, calling `get_structure` to break down
    // the raw data, decoding each field, and then using `from_decoded_fields` to reassemble
    // the fields into the original type.
    fn from_bytes(data: &'a mut [u8]) -> Result<Self, Error> {
        let structure = Self::get_structure(data)?;
        let mut fields = Vec::new();
        let mut tail = data;

        for field in structure {
            let field_size = field.size_hint_(tail, 0)?;
            if field_size > tail.len() {
                return Err(Error::DecodableConversionError);
            }
            let (head, t) = tail.split_at_mut(field_size);
            tail = t;
            fields.push(field.decode(head)?);
        }
        Self::from_decoded_fields(fields)
    }

    // Decodes the type from a reader stream.
    //
    // Instead of working directly with byte slices, this method reads from an I/O source
    // like a file or a network stream. It processes all available data, decodes it, and
    // reconstructs the type.
    #[cfg(not(feature = "no_std"))]
    fn from_reader(reader: &mut impl Read) -> Result<Self, Error> {
        let mut data = Vec::new();
        reader.read_to_end(&mut data)?;

        let structure = Self::get_structure(&data[..])?;

        let mut fields = Vec::new();
        let mut reader = Cursor::new(data);

        for field in structure {
            fields.push(field.from_reader(&mut reader)?);
        }
        Self::from_decoded_fields(fields)
    }
}

/// Enum representing primitive data markers.
///
/// These markers are used to identify primitive types such as integers, booleans, and byte arrays.
/// Each variant represents a specific type and is used during decoding to interpret raw data correctly.
#[derive(Debug, Clone, Copy)]
pub enum PrimitiveMarker {
    U8,
    U16,
    Bool,
    U24,
    U256,
    ShortTxId,
    Signature,
    U32,
    U32AsRef,
    F32,
    U64,
    B032,
    B0255,
    B064K,
    B016M,
}

/// Enum representing field markers used to describe data structure.
///
/// A `FieldMarker` can either be a primitive or a nested structure. The marker helps the decoder
/// understand the layout and type of each field in the data, guiding the decoding process.
#[derive(Debug, Clone)]
pub enum FieldMarker {
    Primitive(PrimitiveMarker),
    Struct(Vec<FieldMarker>),
}

/// Trait that provides a mechanism to retrieve the marker associated with a data field.
///
/// This trait defines a method for getting the marker that represents the structure or
/// type of a given field. It is used to assist in decoding by indicating how to interpret
/// the data.
pub trait GetMarker {
    fn get_marker() -> FieldMarker;
}

/// Represents a decoded primitive data type.
///
/// After decoding, the raw data is transformed into one of these variants, which represent
/// standard primitive types like integers or binary arrays. The decoder uses these values to
/// build the final structure of the message.
#[derive(Debug)]
pub enum DecodablePrimitive<'a> {
    U8(u8),
    U16(u16),
    Bool(bool),
    U24(U24),
    U256(U256<'a>),
    ShortTxId(ShortTxId<'a>),
    Signature(Signature<'a>),
    U32(u32),
    U32AsRef(U32AsRef<'a>),
    F32(f32),
    U64(u64),
    B032(B032<'a>),
    B0255(B0255<'a>),
    B064K(B064K<'a>),
    B016M(B016M<'a>),
}

/// Represents a decoded field, which may be primitive or a nested structure.
///
/// Once the raw data is decoded, it is either classified as a primitive (e.g., integer, boolean)
/// or a structure, which may itself contain multiple decoded fields. This type encapsulates that
/// distinction.
#[derive(Debug)]
pub enum DecodableField<'a> {
    Primitive(DecodablePrimitive<'a>),
    Struct(Vec<DecodableField<'a>>),
}

// Provides size hinting for each primitive marker.
//
// This implementation defines how to estimate the size of data represented by a `PrimitiveMarker`.
// This is useful for efficient decoding, allowing the decoder to correctly split raw data into
// fields of the right size.
impl SizeHint for PrimitiveMarker {
    // PrimitiveMarker need introspection to return a size hint. This method is not implementeable
    fn size_hint(_data: &[u8], _offset: usize) -> Result<usize, Error> {
        unimplemented!()
    }

    fn size_hint_(&self, data: &[u8], offset: usize) -> Result<usize, Error> {
        match self {
            Self::U8 => u8::size_hint(data, offset),
            Self::U16 => u16::size_hint(data, offset),
            Self::Bool => bool::size_hint(data, offset),
            Self::U24 => U24::size_hint(data, offset),
            Self::U256 => U256::size_hint(data, offset),
            Self::ShortTxId => ShortTxId::size_hint(data, offset),
            Self::Signature => Signature::size_hint(data, offset),
            Self::U32 => u32::size_hint(data, offset),
            Self::U32AsRef => U32AsRef::size_hint(data, offset),
            Self::F32 => f32::size_hint(data, offset),
            Self::U64 => u64::size_hint(data, offset),
            Self::B032 => B032::size_hint(data, offset),
            Self::B0255 => B0255::size_hint(data, offset),
            Self::B064K => B064K::size_hint(data, offset),
            Self::B016M => B016M::size_hint(data, offset),
        }
    }
}

// Provides size hinting for each field marker, including nested structures.
//
// This method defines how to estimate the size of a field, whether it's a primitive or a
// composite structure. For composite fields, it recursively calculates the total size.
impl SizeHint for FieldMarker {
    // FieldMarker need introspection to return a size hint. This method is not implementeable
    fn size_hint(_data: &[u8], _offset: usize) -> Result<usize, Error> {
        unimplemented!()
    }

    fn size_hint_(&self, data: &[u8], offset: usize) -> Result<usize, Error> {
        match self {
            Self::Primitive(p) => p.size_hint_(data, offset),
            Self::Struct(ps) => {
                let mut size = 0;
                for p in ps {
                    size += p.size_hint_(data, offset + size)?;
                }
                Ok(size)
            }
        }
    }
}

// Implements size hinting for a vector of field markers, summing the size of individual marker.
impl SizeHint for Vec<FieldMarker> {
    // FieldMarker need introspection to return a size hint. This method is not implementeable
    fn size_hint(_data: &[u8], _offset: usize) -> Result<usize, Error> {
        unimplemented!()
    }

    fn size_hint_(&self, data: &[u8], offset: usize) -> Result<usize, Error> {
        let mut size = 0;
        for field in self {
            let field_size = field.size_hint_(data, offset + size)?;
            size += field_size;
        }
        Ok(size)
    }
}

// Converts a `PrimitiveMarker` into a `FieldMarker`.
//
// This conversion allows primitive types to be represented as field markers, which can
// then be used in the decoding process.
impl From<PrimitiveMarker> for FieldMarker {
    fn from(v: PrimitiveMarker) -> Self {
        FieldMarker::Primitive(v)
    }
}

// Attempts to convert a vector of field markers into a single field marker, representing a structure.
//
// This conversion is useful for handling cases where a sequence of field markers is intended
// to represent a composite structure. If the vector is empty, an error is returned.
impl TryFrom<Vec<FieldMarker>> for FieldMarker {
    type Error = crate::Error;

    fn try_from(mut v: Vec<FieldMarker>) -> Result<Self, crate::Error> {
        match v.len() {
            // It shouldn't be possible to call this function with a void Vec but for safety
            // reasons it is implemented with TryFrom and not From if needed should be possible
            // to use From and just panic
            0 => Err(crate::Error::VoidFieldMarker),
            // This is always safe: if v.len is 1 pop can not fail
            1 => Ok(v.pop().unwrap()),
            _ => Ok(FieldMarker::Struct(v)),
        }
    }
}

// Converts a `DecodableField` into a vector of `DecodableField`s.
// If the field is a primitive, it wraps it in a vector.
// If the field is a structure, it returns the nested fields directly.
impl<'a> From<DecodableField<'a>> for Vec<DecodableField<'a>> {
    fn from(v: DecodableField<'a>) -> Self {
        match v {
            DecodableField::Primitive(p) => vec![DecodableField::Primitive(p)],
            DecodableField::Struct(ps) => ps,
        }
    }
}

// Implements the decoding process for a `PrimitiveMarker`.
// Given a slice of data and an offset, this method parses the corresponding data and returns
// a `DecodablePrimitive`. This is the core mechanism for decoding primitive types like integers,
// booleans, and fixed-length byte arrays from raw byte data.
impl PrimitiveMarker {
    // Decodes a primitive value from a byte slice at the given offset, returning the corresponding
    // `DecodablePrimitive`. The specific decoding logic depends on the type of the primitive (e.g., `u8`, `u16`, etc.).
    fn decode<'a>(&self, data: &'a mut [u8], offset: usize) -> DecodablePrimitive<'a> {
        match self {
            Self::U8 => DecodablePrimitive::U8(u8::from_bytes_unchecked(&mut data[offset..])),
            Self::U16 => DecodablePrimitive::U16(u16::from_bytes_unchecked(&mut data[offset..])),
            Self::Bool => DecodablePrimitive::Bool(bool::from_bytes_unchecked(&mut data[offset..])),
            Self::U24 => DecodablePrimitive::U24(U24::from_bytes_unchecked(&mut data[offset..])),
            Self::U256 => DecodablePrimitive::U256(U256::from_bytes_unchecked(&mut data[offset..])),
            Self::ShortTxId => {
                DecodablePrimitive::ShortTxId(ShortTxId::from_bytes_unchecked(&mut data[offset..]))
            }
            Self::Signature => {
                DecodablePrimitive::Signature(Signature::from_bytes_unchecked(&mut data[offset..]))
            }
            Self::U32 => DecodablePrimitive::U32(u32::from_bytes_unchecked(&mut data[offset..])),
            Self::U32AsRef => {
                DecodablePrimitive::U32AsRef(U32AsRef::from_bytes_unchecked(&mut data[offset..]))
            }
            Self::F32 => DecodablePrimitive::F32(f32::from_bytes_unchecked(&mut data[offset..])),
            Self::U64 => DecodablePrimitive::U64(u64::from_bytes_unchecked(&mut data[offset..])),
            Self::B032 => DecodablePrimitive::B032(B032::from_bytes_unchecked(&mut data[offset..])),
            Self::B0255 => {
                DecodablePrimitive::B0255(B0255::from_bytes_unchecked(&mut data[offset..]))
            }
            Self::B064K => {
                DecodablePrimitive::B064K(B064K::from_bytes_unchecked(&mut data[offset..]))
            }
            Self::B016M => {
                DecodablePrimitive::B016M(B016M::from_bytes_unchecked(&mut data[offset..]))
            }
        }
    }

    // Decodes a primitive value from a reader stream, returning the corresponding
    // `DecodablePrimitive`. This is useful when reading data from a file or network socket,
    // where the data is not immediately available as a slice but must be read incrementally.
    #[allow(clippy::wrong_self_convention)]
    #[cfg(not(feature = "no_std"))]
    #[allow(clippy::wrong_self_convention)]
    #[cfg(not(feature = "no_std"))]
    #[allow(clippy::wrong_self_convention)]
    fn from_reader<'a>(&self, reader: &mut impl Read) -> Result<DecodablePrimitive<'a>, Error> {
        match self {
            Self::U8 => Ok(DecodablePrimitive::U8(u8::from_reader_(reader)?)),
            Self::U16 => Ok(DecodablePrimitive::U16(u16::from_reader_(reader)?)),
            Self::Bool => Ok(DecodablePrimitive::Bool(bool::from_reader_(reader)?)),
            Self::U24 => Ok(DecodablePrimitive::U24(U24::from_reader_(reader)?)),
            Self::U256 => Ok(DecodablePrimitive::U256(U256::from_reader_(reader)?)),
            Self::ShortTxId => Ok(DecodablePrimitive::ShortTxId(ShortTxId::from_reader_(
                reader,
            )?)),
            Self::Signature => Ok(DecodablePrimitive::Signature(Signature::from_reader_(
                reader,
            )?)),
            Self::U32 => Ok(DecodablePrimitive::U32(u32::from_reader_(reader)?)),
            Self::U32AsRef => Ok(DecodablePrimitive::U32AsRef(U32AsRef::from_reader_(
                reader,
            )?)),
            Self::F32 => Ok(DecodablePrimitive::F32(f32::from_reader_(reader)?)),
            Self::U64 => Ok(DecodablePrimitive::U64(u64::from_reader_(reader)?)),
            Self::B032 => Ok(DecodablePrimitive::B032(B032::from_reader_(reader)?)),
            Self::B0255 => Ok(DecodablePrimitive::B0255(B0255::from_reader_(reader)?)),
            Self::B064K => Ok(DecodablePrimitive::B064K(B064K::from_reader_(reader)?)),
            Self::B016M => Ok(DecodablePrimitive::B016M(B016M::from_reader_(reader)?)),
        }
    }
}

impl<'a> GetSize for DecodablePrimitive<'a> {
    fn get_size(&self) -> usize {
        match self {
            DecodablePrimitive::U8(v) => v.get_size(),
            DecodablePrimitive::U16(v) => v.get_size(),
            DecodablePrimitive::Bool(v) => v.get_size(),
            DecodablePrimitive::U24(v) => v.get_size(),
            DecodablePrimitive::U256(v) => v.get_size(),
            DecodablePrimitive::ShortTxId(v) => v.get_size(),
            DecodablePrimitive::Signature(v) => v.get_size(),
            DecodablePrimitive::U32(v) => v.get_size(),
            DecodablePrimitive::U32AsRef(v) => v.get_size(),
            DecodablePrimitive::F32(v) => v.get_size(),
            DecodablePrimitive::U64(v) => v.get_size(),
            DecodablePrimitive::B032(v) => v.get_size(),
            DecodablePrimitive::B0255(v) => v.get_size(),
            DecodablePrimitive::B064K(v) => v.get_size(),
            DecodablePrimitive::B016M(v) => v.get_size(),
        }
    }
}

// Implements the decoding functionality for a `FieldMarker`.
// Depending on whether the field is primitive or structured, this method decodes the corresponding data.
// If the field is a structure, it recursively decodes each nested field and returns the resulting
// `DecodableField`.
impl FieldMarker {
    pub(crate) fn decode<'a>(&self, data: &'a mut [u8]) -> Result<DecodableField<'a>, Error> {
        match self {
            Self::Primitive(p) => Ok(DecodableField::Primitive(p.decode(data, 0))),
            Self::Struct(ps) => {
                let mut decodeds = Vec::new();
                let mut tail = data;
                for p in ps {
                    let field_size = p.size_hint_(tail, 0)?;
                    let (head, t) = tail.split_at_mut(field_size);
                    tail = t;
                    decodeds.push(p.decode(head)?);
                }
                Ok(DecodableField::Struct(decodeds))
            }
        }
    }

    #[allow(clippy::wrong_self_convention)]
    #[cfg(not(feature = "no_std"))]
    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn from_reader<'a>(
        &self,
        reader: &mut impl Read,
    ) -> Result<DecodableField<'a>, Error> {
        match self {
            Self::Primitive(p) => Ok(DecodableField::Primitive(p.from_reader(reader)?)),
            Self::Struct(ps) => {
                let mut decodeds = Vec::new();
                for p in ps {
                    decodeds.push(p.from_reader(reader)?);
                }
                Ok(DecodableField::Struct(decodeds))
            }
        }
    }
}
