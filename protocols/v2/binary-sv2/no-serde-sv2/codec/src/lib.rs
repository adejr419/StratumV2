//! This module defines types, encodings, and conversions between Serde and SV2 protocols,
//! providing abstractions to facilitate encoding, decoding, and error handling of SV2 data types.
//!
//! # Overview
//!
//! SV2 (Stratum V2) is a protocol used in cryptocurrency mining pools. This module allows the conversion
//! between various Rust types and SV2-specific data formats, which are used for efficient network communication.
//! It also provides utilities to encode and decode data types according to the SV2 specifications.
//!
//! ## Type Mappings
//! The following table illustrates how standard Rust types map to their SV2 counterparts:
//!
//! ```txt
//! SERDE    <-> Sv2
//! bool     <-> BOOL
//! u8       <-> U8
//! u16      <-> U16
//! U24      <-> U24
//! u32      <-> U32
//! f32      <-> F32     // Not in the spec, but used
//! u64      <-> U64     // Not in the spec, but used
//! U256     <-> U256
//! Str0255  <-> STRO_255
//! Signature<-> SIGNATURE
//! B032     <-> B0_32   // Not in the spec, but used
//! B0255    <-> B0_255
//! B064K    <-> B0_64K
//! B016M    <-> B0_16M
//! [u8]     <-> BYTES
//! Pubkey   <-> PUBKEY
//! Seq0255  <-> SEQ0_255[T]
//! Seq064K  <-> SEQ0_64K[T]
//! ```
//!
//! # Encoding & Decoding
//!
//! The module provides functions to encode and decode data types using the SV2 encoding scheme. This is critical for ensuring
//! that data is correctly serialized for communication over the network.
//!
//! - **to_bytes**: Encodes an SV2 data type into a byte vector.
//! - **to_writer**: Encodes an SV2 data type into a byte slice.
//! - **from_bytes**: Decodes an SV2-encoded byte slice into the specified data type.
//!
//! # Error Handling
//!
//! The module defines an `Error` enum for handling various failure conditions during encoding, decoding, and data manipulation.
//! Common errors include:
//! - Out of bounds accesses
//! - Size mismatches during encoding/decoding
//! - Invalid data representations (e.g., non-boolean values interpreted as booleans)
//!
//! # Cross-Language Interoperability
//!
//! To support foreign function interface (FFI) use cases, the module includes `CError` and `CVec` types that represent SV2 data and errors in a format
//! suitable for cross-language compatibility.
//!
//! # Features
//!
//! The module supports optional features like `no_std` for environments without standard library support. Error types are conditionally compiled
//! to work with or without `std`.
//!
//! ## Conditional Compilation
//! - When the `no_std` feature is enabled, I/O-related errors use a simplified `IoError` representation.
//! - Standard I/O errors (`std::io::Error`) are used when `no_std` is disabled.
//!
//! # FFI Interoperability
//!
//! This module provides several utilities for FFI (Foreign Function Interface) to facilitate the passing of data between Rust and other languages.
//! These utilities include:
//! - `CVec`: A representation of a byte vector that can be safely passed between C and Rust.
//! - `CError`: A C-compatible version of the error type.
//! - `CVec2`: A struct to manage collections of `CVec` objects across FFI boundaries.
//!
//! These structures allow easy integration of SV2-related functionality into cross-language projects.

#![cfg_attr(feature = "no_std", no_std)]

#[cfg(not(feature = "no_std"))]
use std::io::{Error as E, ErrorKind};

mod codec;
mod datatypes;
pub use datatypes::{
    PubKey, Seq0255, Seq064K, ShortTxId, Signature, Str0255, Sv2DataType, Sv2Option, U32AsRef,
    B016M, B0255, B032, B064K, U24, U256,
};

pub use crate::codec::{
    decodable::{Decodable, GetMarker},
    encodable::{Encodable, EncodableField},
    Fixed, GetSize, SizeHint,
};

use alloc::vec::Vec;

/// Converts the provided SV2 data type to a byte vector based on the SV2 encoding format.
#[allow(clippy::wrong_self_convention)]
pub fn to_bytes<T: Encodable + GetSize>(src: T) -> Result<Vec<u8>, Error> {
    let mut result = vec![0_u8; src.get_size()];
    src.to_bytes(&mut result)?;
    Ok(result)
}

/// Encodes the SV2 data type to the provided byte slice.
#[allow(clippy::wrong_self_convention)]
pub fn to_writer<T: Encodable>(src: T, dst: &mut [u8]) -> Result<(), Error> {
    src.to_bytes(dst)?;
    Ok(())
}

/// Decodes an SV2-encoded byte slice into the specified data type.
pub fn from_bytes<'a, T: Decodable<'a>>(data: &'a mut [u8]) -> Result<T, Error> {
    T::from_bytes(data)
}

pub mod decodable {
    pub use crate::codec::decodable::{Decodable, DecodableField, FieldMarker};
    //pub use crate::codec::decodable::PrimitiveMarker;
}

pub mod encodable {
    pub use crate::codec::encodable::{Encodable, EncodableField, EncodablePrimitive};
}

#[macro_use]
extern crate alloc;

/// Error types used within the protocol library to indicate various failure conditions.
///
/// - `OutOfBound`: Indicates an attempt to read beyond a valid range.
/// - `NotABool(u8)`: Raised when a non-binary value is interpreted as a boolean.
/// - `WriteError(usize, usize)`: Occurs when an unexpected size mismatch arises during a write operation, specifying expected and actual sizes.
/// - `U24TooBig(u32)`: Signifies an overflow condition where a `u32` exceeds the maximum allowable `u24` value.
/// - `InvalidSignatureSize(usize)`: Reports a size mismatch for a signature, such as when it does not match the expected size.
/// - `InvalidU256(usize)`: Raised when a `u256` value is invalid, typically due to size discrepancies.
/// - `InvalidU24(u32)`: Indicates an invalid `u24` representation.
/// - `InvalidB0255Size(usize)`, `InvalidB064KSize(usize)`, `InvalidB016MSize(usize)`: Indicate that a byte array exceeds the maximum allowed size for `B0255`, `B064K`, or `B016M` types, respectively.
/// - `InvalidSeq0255Size(usize)`: Raised when a sequence size exceeds `0255`.
/// - `NonPrimitiveTypeCannotBeEncoded`: Error indicating an attempt to encode a complex type as a primitive.
/// - `PrimitiveConversionError`: Generic conversion error related to primitive types.
/// - `DecodableConversionError`: Error occurring during decoding due to conversion issues.
/// - `UnInitializedDecoder`: Error triggered when a decoder is used without initialization.
/// - `IoError`: Represents I/O-related errors, compatible with `no_std` mode where specific error types may vary.
/// - `ReadError(usize, usize)`: Raised when an unexpected mismatch occurs during read operations, specifying expected and actual read sizes.
/// - `VoidFieldMarker`: Used as a marker error for fields that should remain void or empty.
/// - `ValueExceedsMaxSize(bool, usize, usize, usize, Vec<u8>, usize)`: Signifies a value overflow based on protocol restrictions, containing details about fixed/variable size, maximum size allowed, and the offending value details.
/// - `SeqExceedsMaxSize`: Triggered when a sequence type (`Seq0255`, `Seq064K`) exceeds its maximum allowable size.
/// - `NoDecodableFieldPassed`: Raised when no valid decodable field is provided during decoding.
/// - `ValueIsNotAValidProtocol(u8)`: Error for protocol-specific invalid values.
/// - `UnknownMessageType(u8)`: Raised when an unsupported or unknown message type is encountered.
/// - `Sv2OptionHaveMoreThenOneElement(u8)`: Indicates a protocol constraint violation where `Sv2Option` unexpectedly contains multiple elements.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Error {
    OutOfBound,
    NotABool(u8),
    /// -> (expected size, actual size)
    WriteError(usize, usize),
    U24TooBig(u32),
    InvalidSignatureSize(usize),
    InvalidU256(usize),
    InvalidU24(u32),
    InvalidB0255Size(usize),
    InvalidB064KSize(usize),
    InvalidB016MSize(usize),
    InvalidSeq0255Size(usize),
    /// Error when trying to encode a non-primitive data type
    NonPrimitiveTypeCannotBeEncoded,
    PrimitiveConversionError,
    DecodableConversionError,
    UnInitializedDecoder,
    #[cfg(not(feature = "no_std"))]
    IoError(E),
    #[cfg(feature = "no_std")]
    IoError,
    ReadError(usize, usize),
    VoidFieldMarker,
    /// Error when `Inner` type value exceeds max size.
    /// (ISFIXED, SIZE, HEADERSIZE, MAXSIZE, bad value vec, bad value length)
    ValueExceedsMaxSize(bool, usize, usize, usize, Vec<u8>, usize),
    /// Error when sequence value (`Seq0255`, `Seq064K`) exceeds max size
    SeqExceedsMaxSize,
    NoDecodableFieldPassed,
    ValueIsNotAValidProtocol(u8),
    UnknownMessageType(u8),
    Sv2OptionHaveMoreThenOneElement(u8),
}

#[cfg(not(feature = "no_std"))]
impl From<E> for Error {
    fn from(v: E) -> Self {
        match v.kind() {
            ErrorKind::UnexpectedEof => Error::OutOfBound,
            _ => Error::IoError(v),
        }
    }
}

/// `CError` is a foreign function interface (FFI)-compatible version of the `Error` enum to facilitate cross-language compatibility.
#[repr(C)]
#[derive(Debug)]
pub enum CError {
    OutOfBound,
    NotABool(u8),
    /// -> (expected size, actual size)
    WriteError(usize, usize),
    U24TooBig(u32),
    InvalidSignatureSize(usize),
    InvalidU256(usize),
    InvalidU24(u32),
    InvalidB0255Size(usize),
    InvalidB064KSize(usize),
    InvalidB016MSize(usize),
    InvalidSeq0255Size(usize),
    /// Error when trying to encode a non-primitive data type
    NonPrimitiveTypeCannotBeEncoded,
    PrimitiveConversionError,
    DecodableConversionError,
    UnInitializedDecoder,
    #[cfg(not(feature = "no_std"))]
    IoError(E),
    #[cfg(feature = "no_std")]
    IoError,
    ReadError(usize, usize),
    VoidFieldMarker,
    /// Error when `Inner` type value exceeds max size.
    /// (ISFIXED, SIZE, HEADERSIZE, MAXSIZE, bad value vec, bad value length)
    ValueExceedsMaxSize(bool, usize, usize, usize, CVec, usize),
    /// Error when sequence value (`Seq0255`, `Seq064K`) exceeds max size
    SeqExceedsMaxSize,
    NoDecodableFieldPassed,
    ValueIsNotAValidProtocol(u8),
    UnknownMessageType(u8),
    Sv2OptionHaveMoreThenOneElement(u8),
}

impl From<Error> for CError {
    fn from(e: Error) -> CError {
        match e {
            Error::OutOfBound => CError::OutOfBound,
            Error::NotABool(u) => CError::NotABool(u),
            Error::WriteError(u1, u2) => CError::WriteError(u1, u2),
            Error::U24TooBig(u) => CError::U24TooBig(u),
            Error::InvalidSignatureSize(u) => CError::InvalidSignatureSize(u),
            Error::InvalidU256(u) => CError::InvalidU256(u),
            Error::InvalidU24(u) => CError::InvalidU24(u),
            Error::InvalidB0255Size(u) => CError::InvalidB0255Size(u),
            Error::InvalidB064KSize(u) => CError::InvalidB064KSize(u),
            Error::InvalidB016MSize(u) => CError::InvalidB016MSize(u),
            Error::InvalidSeq0255Size(u) => CError::InvalidSeq0255Size(u),
            Error::NonPrimitiveTypeCannotBeEncoded => CError::NonPrimitiveTypeCannotBeEncoded,
            Error::PrimitiveConversionError => CError::PrimitiveConversionError,
            Error::DecodableConversionError => CError::DecodableConversionError,
            Error::UnInitializedDecoder => CError::UnInitializedDecoder,
            #[cfg(not(feature = "no_std"))]
            Error::IoError(e) => CError::IoError(e),
            #[cfg(feature = "no_std")]
            Error::IoError => CError::IoError,
            Error::ReadError(u1, u2) => CError::ReadError(u1, u2),
            Error::VoidFieldMarker => CError::VoidFieldMarker,
            Error::ValueExceedsMaxSize(isfixed, size, headersize, maxsize, bad_value, bad_len) => {
                let bv1: &[u8] = bad_value.as_ref();
                let bv: CVec = bv1.into();
                CError::ValueExceedsMaxSize(isfixed, size, headersize, maxsize, bv, bad_len)
            }
            Error::SeqExceedsMaxSize => CError::SeqExceedsMaxSize,
            Error::NoDecodableFieldPassed => CError::NoDecodableFieldPassed,
            Error::ValueIsNotAValidProtocol(u) => CError::ValueIsNotAValidProtocol(u),
            Error::UnknownMessageType(u) => CError::UnknownMessageType(u),
            Error::Sv2OptionHaveMoreThenOneElement(u) => CError::Sv2OptionHaveMoreThenOneElement(u),
        }
    }
}

impl Drop for CError {
    fn drop(&mut self) {
        match self {
            Self::OutOfBound => (),
            Self::NotABool(_) => (),
            Self::WriteError(_, _) => (),
            Self::U24TooBig(_) => (),
            Self::InvalidSignatureSize(_) => (),
            Self::InvalidU256(_) => (),
            Self::InvalidU24(_) => (),
            Self::InvalidB0255Size(_) => (),
            Self::InvalidB064KSize(_) => (),
            Self::InvalidB016MSize(_) => (),
            Self::InvalidSeq0255Size(_) => (),
            Self::NonPrimitiveTypeCannotBeEncoded => (),
            Self::PrimitiveConversionError => (),
            Self::DecodableConversionError => (),
            Self::UnInitializedDecoder => (),
            #[cfg(not(feature = "no_std"))]
            Self::IoError(_) => (),
            #[cfg(feature = "no_std")]
            Self::IoError => (),
            Self::ReadError(_, _) => (),
            Self::VoidFieldMarker => (),
            Self::ValueExceedsMaxSize(_, _, _, _, cvec, _) => free_vec(cvec),
            Self::SeqExceedsMaxSize => (),
            Self::NoDecodableFieldPassed => (),
            Self::ValueIsNotAValidProtocol(_) => (),
            Self::UnknownMessageType(_) => (),
            Self::Sv2OptionHaveMoreThenOneElement(_) => (),
        };
    }
}

/// Vec<u8> is used as the Sv2 type Bytes
impl GetSize for Vec<u8> {
    fn get_size(&self) -> usize {
        self.len()
    }
}

// Only needed for implement encodable for Frame never called
impl<'a> From<Vec<u8>> for EncodableField<'a> {
    fn from(_v: Vec<u8>) -> Self {
        unreachable!()
    }
}

#[cfg(feature = "with_buffer_pool")]
impl<'a> From<buffer_sv2::Slice> for EncodableField<'a> {
    fn from(_v: buffer_sv2::Slice) -> Self {
        unreachable!()
    }
}

/// A struct to facilitate transferring a `Vec<u8>` across FFI boundaries.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CVec {
    data: *mut u8,
    len: usize,
    capacity: usize,
}

impl CVec {
    /// Returns a mutable slice of the contained data.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the data pointed to by `self.data`
    /// remains valid for the duration of the returned slice.
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe { core::slice::from_raw_parts_mut(self.data, self.len) }
    }

    /// Used when we need to fill a buffer allocated in rust from C.
    ///
    /// # Safety
    ///
    /// This function construct a CVec without taking ownership of the pointed buffer so if the
    /// owner drop them the CVec will point to garbage.
    #[allow(clippy::wrong_self_convention)]
    pub fn as_shared_buffer(v: &mut [u8]) -> Self {
        let (data, len) = (v.as_mut_ptr(), v.len());
        Self {
            data,
            len,
            capacity: len,
        }
    }
}

impl From<&[u8]> for CVec {
    fn from(v: &[u8]) -> Self {
        let mut buffer: Vec<u8> = vec![0; v.len()];
        buffer.copy_from_slice(v);

        // Get the length, first, then the pointer (doing it the other way around **currently**
        // doesn't cause UB, but it may be unsound due to unclear (to me, at least) guarantees of
        // the std lib)
        let len = buffer.len();
        let ptr = buffer.as_mut_ptr();
        core::mem::forget(buffer);

        CVec {
            data: ptr,
            len,
            capacity: len,
        }
    }
}

/// Creates a `CVec` from a buffer that was allocated in C.
///
/// # Safety
/// The caller must ensure that the buffer is valid and that
/// the data length does not exceed the allocated size.
#[no_mangle]
pub unsafe extern "C" fn cvec_from_buffer(data: *const u8, len: usize) -> CVec {
    let input = core::slice::from_raw_parts(data, len);

    let mut buffer: Vec<u8> = vec![0; len];
    buffer.copy_from_slice(input);

    // Get the length, first, then the pointer (doing it the other way around **currently** doesn't
    // cause UB, but it may be unsound due to unclear (to me, at least) guarantees of the std lib)
    let len = buffer.len();
    let ptr = buffer.as_mut_ptr();
    core::mem::forget(buffer);

    CVec {
        data: ptr,
        len,
        capacity: len,
    }
}

/// A struct to manage a collection of `CVec` objects across FFI boundaries.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CVec2 {
    data: *mut CVec,
    len: usize,
    capacity: usize,
}

impl CVec2 {
    pub fn as_mut_slice(&mut self) -> &mut [CVec] {
        unsafe { core::slice::from_raw_parts_mut(self.data, self.len) }
    }
}
impl From<CVec2> for Vec<CVec> {
    fn from(v: CVec2) -> Self {
        unsafe { Vec::from_raw_parts(v.data, v.len, v.capacity) }
    }
}

/// Frees the underlying memory of a `CVec`.
pub fn free_vec(buf: &mut CVec) {
    let _: Vec<u8> = unsafe { Vec::from_raw_parts(buf.data, buf.len, buf.capacity) };
}

/// Frees the underlying memory of a `CVec2` and all its elements.
pub fn free_vec_2(buf: &mut CVec2) {
    let vs: Vec<CVec> = unsafe { Vec::from_raw_parts(buf.data, buf.len, buf.capacity) };
    for mut s in vs {
        free_vec(&mut s)
    }
}

impl<'a, const A: bool, const B: usize, const C: usize, const D: usize>
    From<datatypes::Inner<'a, A, B, C, D>> for CVec
{
    fn from(v: datatypes::Inner<'a, A, B, C, D>) -> Self {
        let (ptr, len, cap): (*mut u8, usize, usize) = match v {
            datatypes::Inner::Ref(inner) => {
                // Data is copied in a vector that then will be forgetted from the allocator,
                // cause the owner of the data is going to be dropped by rust
                let mut inner: Vec<u8> = inner.into();

                // Get the length, first, then the pointer (doing it the other way around
                // **currently** doesn't cause UB, but it may be unsound due to unclear (to me, at
                // least) guarantees of the std lib)
                let len = inner.len();
                let cap = inner.capacity();
                let ptr = inner.as_mut_ptr();
                core::mem::forget(inner);

                (ptr, len, cap)
            }
            datatypes::Inner::Owned(mut inner) => {
                // Get the length, first, then the pointer (doing it the other way around
                // **currently** doesn't cause UB, but it may be unsound due to unclear (to me, at
                // least) guarantees of the std lib)
                let len = inner.len();
                let cap = inner.capacity();
                let ptr = inner.as_mut_ptr();
                core::mem::forget(inner);

                (ptr, len, cap)
            }
        };
        Self {
            data: ptr,
            len,
            capacity: cap,
        }
    }
}

/// Initializes an empty `CVec2`.
///
/// # Safety
/// The caller is responsible for freeing the `CVec2` when it is no longer needed.
#[no_mangle]
pub unsafe extern "C" fn init_cvec2() -> CVec2 {
    let mut buffer = Vec::<CVec>::new();

    // Get the length, first, then the pointer (doing it the other way around **currently** doesn't
    // cause UB, but it may be unsound due to unclear (to me, at least) guarantees of the std lib)
    let len = buffer.len();
    let ptr = buffer.as_mut_ptr();
    core::mem::forget(buffer);

    CVec2 {
        data: ptr,
        len,
        capacity: len,
    }
}

/// Adds a `CVec` to a `CVec2`.
///
/// # Safety
/// The caller must ensure no duplicate `CVec`s are added, as duplicates may
/// lead to double-free errors when the message is dropped.
#[no_mangle]
pub unsafe extern "C" fn cvec2_push(cvec2: &mut CVec2, cvec: CVec) {
    let mut buffer: Vec<CVec> = Vec::from_raw_parts(cvec2.data, cvec2.len, cvec2.capacity);
    buffer.push(cvec);

    let len = buffer.len();
    let ptr = buffer.as_mut_ptr();
    core::mem::forget(buffer);

    cvec2.data = ptr;
    cvec2.len = len;
    cvec2.capacity = len;
}

impl<'a, T: Into<CVec>> From<Seq0255<'a, T>> for CVec2 {
    fn from(v: Seq0255<'a, T>) -> Self {
        let mut v: Vec<CVec> = v.0.into_iter().map(|x| x.into()).collect();
        // Get the length, first, then the pointer (doing it the other way around **currently**
        // doesn't cause UB, but it may be unsound due to unclear (to me, at least) guarantees of
        // the std lib)
        let len = v.len();
        let capacity = v.capacity();
        let data = v.as_mut_ptr();
        core::mem::forget(v);
        Self {
            data,
            len,
            capacity,
        }
    }
}
impl<'a, T: Into<CVec>> From<Seq064K<'a, T>> for CVec2 {
    fn from(v: Seq064K<'a, T>) -> Self {
        let mut v: Vec<CVec> = v.0.into_iter().map(|x| x.into()).collect();
        // Get the length, first, then the pointer (doing it the other way around **currently**
        // doesn't cause UB, but it may be unsound due to unclear (to me, at least) guarantees of
        // the std lib)
        let len = v.len();
        let capacity = v.capacity();
        let data = v.as_mut_ptr();
        core::mem::forget(v);
        Self {
            data,
            len,
            capacity,
        }
    }
}

/// Exported FFI functions for interoperability with C code.
#[no_mangle]
pub extern "C" fn _c_export_u24(_a: U24) {}
#[no_mangle]
pub extern "C" fn _c_export_cvec(_a: CVec) {}
#[no_mangle]
pub extern "C" fn _c_export_cvec2(_a: CVec2) {}
