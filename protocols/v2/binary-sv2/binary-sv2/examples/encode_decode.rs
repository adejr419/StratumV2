pub use binary_codec_sv2::{self, Decodable as Deserialize, Encodable as Serialize, *};
pub use derive_codec_sv2::{Decodable as Deserialize, Encodable as Serialize};
use core::convert::TryInto;

#[derive(Clone, Deserialize, Serialize, PartialEq, Debug)]
struct Test {
    a: u32,
    b: u8,
    c: U24,
}

fn main() {
    let expected = Test {
        a: 456,
        b: 9,
        c: 67_u32.try_into().unwrap(),
    };

    let mut bytes = to_bytes(expected.clone()).unwrap();

    let deserialized: Test = from_bytes(&mut bytes[..]).unwrap();

    assert_eq!(deserialized, expected);
}