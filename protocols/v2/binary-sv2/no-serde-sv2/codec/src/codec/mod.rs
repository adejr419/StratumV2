/// Provides traits and utilities for encoding and decoding data at a low level,
/// prioritizing efficiency and memory safety by operating directly on slices rather than
/// relying on `Read` or `Write` streams.
///
/// ## Overview
///
/// Optimized for performance, this module directly manipulates byte slices, avoiding
/// the overhead of stream-based I/O. It uses memory-efficient techniques like obtaining
/// pointers to original data instead of copying it. Enums are avoided for decoding, as
/// each message type can be identified by its numeric identifier, streamlining the process.
///
/// ### Key Components
///
/// - **Traits**: Defines core traits (`SizeHint`, `GetSize`, `Fixed`, `Variable`) that establish
///   a consistent interface for encoding and decoding operations.
/// - **Buffer Management**: With the `with_buffer_pool` feature enabled, the `Slice` type from
///   `buffer_sv2` is included, supporting memory pooling and efficient slice handling for high-performance
///   buffer management scenarios.
///
/// ### Traits Overview
///
/// - **`SizeHint`**: Estimates the size of a decodable type, useful for variable-length data where the
///   size must be determined dynamically.
/// - **`GetSize`**: Provides the exact size of an encodable type in bytes, crucial for buffer allocation.
/// - **`Fixed`**: Marks types with a compile-time constant size, simplifying encoding and decoding.
/// - **`Variable`**: For types with dynamic sizes, manages size variability and calculates inner sizes.
///
/// ## Build Options
///
/// - **`no_std` Compatibility**: This module can be compiled without the standard library for constrained
///   environments. Some methods and traits are conditionally available when `std` is included.
/// - **`with_buffer_pool`**: When enabled, includes the `Slice` type for managing pooled memory slices,
///   improving memory handling and efficiency in high-performance scenarios.
///
/// ## Detailed Trait Descriptions
///
/// ### `SizeHint`
/// Defines methods to calculate the size of encoded data for types with variable sizes.
/// - **`size_hint`**: Returns the total size of the encoded data for raw data and an offset.
/// - **`size_hint_`**: Returns the size for a specific instance, offering flexibility.
///
/// ### `GetSize`
/// Provides a `get_size` method that returns the exact size in bytes of an encodable type.
///
/// ### `Fixed`
/// For types with a fixed size, this trait defines a constant `SIZE`, simplifying work with fixed-size types.
///
/// ### `Variable`
/// Types with variable sizes implement this trait, providing constants (`HEADER_SIZE`, `MAX_SIZE`) and methods
/// for size management and inner size calculation.
///
/// ## Summary
///
/// This module supports efficient, low-level encoding and decoding by operating directly on slices, avoiding excess
/// data copying. It offers capabilities for both fixed and variable-sized data, making it versatile for a wide range
/// of encoding tasks.
use crate::Error;
pub mod decodable;
pub mod encodable;
mod impls;
#[cfg(feature = "with_buffer_pool")]
use buffer_sv2::Slice;

use alloc::vec::Vec;

/// The `SizeHint` trait provides a mechanism to return the encoded bytes size of a decodable type.
///
/// It defines two methods for retrieving the size of an encoded message:
///
/// - `size_hint` is a static method that takes the raw data and an offset and returns the total
///     size of the encoded message. This is particularly useful for types where the encoded size
///     may vary based on the contents of the data, and we need to calculate how much space is
///     required for decoding.
/// - `size_hint_` is an instance method that performs the same function but allows the size to be
///     be determined with respect to the current instance of the type.
///
/// These methods are crucial in decoding scenarios where the full size of the message
/// is not immediately known, helping to determine how many bytes need to be read.
pub trait SizeHint {
    /// `size_hint` is a static method that takes the raw data and an offset and returns the total
    /// size of the encoded message. This is particularly useful for types where the encoded size
    /// may vary based on the contents of the data, and we need to calculate how much space is
    /// required for decoding.
    fn size_hint(data: &[u8], offset: usize) -> Result<usize, Error>;
    /// `size_hint_` is an instance method that performs the same function but allows the size to be
    /// be determined with respect to the current instance of the type. 
    fn size_hint_(&self, data: &[u8], offset: usize) -> Result<usize, Error>;
}

/// The `GetSize` trait returns the total size in bytes of an encodable type.
///
/// It provides a single method, `get_size`, which returns the total size of the type
/// in bytes.
pub trait GetSize {
    /// `get_size` returns total size of the type in bytes.
    fn get_size(&self) -> usize;
}

#[cfg(feature = "with_buffer_pool")]
impl GetSize for Slice {
    // Provides the total size of a `Slice` by returning its length.
    //
    // This implementation for the `Slice` type returns the number of bytes in the
    // slice, which represents its total size when encoded.
    fn get_size(&self) -> usize {
        self.len()
    }
}
/// The `Fixed` trait is implemented by all primitives with a constant, fixed size.
///
/// Types implementing this trait must define the constant `SIZE`, representing the
/// fixed number of bytes needed to encode or decode the type. This trait is used for
/// types that have a know size at compile time , such as integers, fixed-size arrays, etc.
pub trait Fixed {
    ///the constant `SIZE`, represents the fixed number of bytes needed to encode or decode the type.
    const SIZE: usize;
}

/// The `Variable` trait is designed for types that have variable size when encoded.
///
/// Types implementing this trait provide the following:
///
/// - `HEADER_SIZE`: The size of the header in bytes. This header often contains metadata like
///     the length of the variables-sized data.
/// - `MAX_SIZE`: The maximum allowed size for this type. This value is essential in
///     ensuring that dynamically sized data does not exceed defined limits.
///
/// The trait also includes methods to calculate the inner size of the data (`inner_size`) and
/// to return the header as a byte vector (`get_header`). These methods are essential for
/// managing dynamically sized types in scenarios like serialization and encoding.
pub trait Variable {
    const HEADER_SIZE: usize;
    //const ELEMENT_SIZE: usize;
    const MAX_SIZE: usize;

    fn inner_size(&self) -> usize;

    /// Retrieves the header as a byte vector. This header typically contains information
    /// about the size or type of the data that follows.
    fn get_header(&self) -> Vec<u8>;
}

impl<T: Fixed> SizeHint for T {
    // Provides the size hind for a fixed-size type
    //
    // Since the type implementing `Fixed` has a constant size, the `size_hint` method
    // simply returns the fixed size, making it easy to predict the size of the encoded data.
    fn size_hint(_data: &[u8], _offset: usize) -> Result<usize, Error> {
        Ok(Self::SIZE)
    }

    //Instance-based size hint for a fixed-size type.
    //
    // Similar to the static `size_hint_`, this method returns the constant size for
    // the specific instance of the type.
    fn size_hint_(&self, _: &[u8], _offset: usize) -> Result<usize, Error> {
        Ok(Self::SIZE)
    }
}

impl<T: Fixed> GetSize for T {
    // Returns the size of the fixed-size type.
    //
    // As the type implements `Fixed`, this method directly returns the constant `SIZE`
    // associated with the type. This is useful when encoding or decoding to know exactly
    // how much space the type occupies in memory or when serialized.
    fn get_size(&self) -> usize {
        Self::SIZE
    }
}
