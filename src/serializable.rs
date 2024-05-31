use std::fmt::Debug;

pub trait Serializable {
    type Error: Debug;
    type DecodeError: Debug;
    type Config;

    fn config() -> Self::Config;
    fn encode_to_string(&self, config: &Self::Config) -> Result<String, Self::Error>;
    fn decode_from_string(
        val: &String,
        config: &Self::Config,
    ) -> Result<(Self, usize), Self::DecodeError>
    where
        Self: Sized;
}
#[derive(Debug)]
pub enum EncodeError {
    Bincode(bincode::error::EncodeError),
    Flate2(std::io::Error),
}

impl std::fmt::Display for EncodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            EncodeError::Bincode(ref err) => write!(f, "Bincode error: {}", err),
            EncodeError::Flate2(ref err) => write!(f, "Flate2 error: {}", err),
        }
    }
}

impl From<bincode::error::EncodeError> for EncodeError {
    fn from(error: bincode::error::EncodeError) -> Self {
        EncodeError::Bincode(error)
    }
}

impl From<std::io::Error> for EncodeError {
    fn from(error: std::io::Error) -> Self {
        EncodeError::Flate2(error)
    }
}
pub trait Serializable2 {
    type EncodeError: Debug;
    type DecodeError: Debug;
    type Config;

    fn config() -> Self::Config;
    fn serialize(&self, config: &Self::Config) -> Result<Vec<u8>, Self::EncodeError>;
    fn deserialize(
        val: &Vec<u8>,
        config: &Self::Config,
    ) -> Result<(Self, usize), Self::DecodeError>
    where
        Self: Sized;
}

#[cfg(test)]
mod serializer_tests {
    use crate::serializable::EncodeError;
    use crate::serializable::Serializable;
    use crate::serializable::Serializable2;
    use bincode::{Decode, Encode};
    use derive::Serializable;
    use derive::Serializable2;
    use rutie::{NilClass, Object, RString, VM};
    extern crate flate2;
    use flate2::Compression;
    use std::io::Write;

    #[derive(Serializable)]
    #[encode_decode(lan = "ruby")]
    pub struct RubyObject {
        pub value: rutie::types::Value,
    }

    impl RubyObject {
        fn new() -> Self {
            Self {
                value: NilClass::new().value(),
            }
        }
    }

    #[derive(Serializable2)]
    #[encode_decode(lan = "ruby")]
    pub struct RubyObject2 {
        pub value: rutie::types::Value,
    }

    impl RubyObject2 {
        fn new() -> Self {
            Self {
                value: NilClass::new().value(),
            }
        }
    }

    #[derive(Encode, Decode, Serializable)]
    pub struct Struct {
        a: bool,
    }

    impl Struct {
        pub fn new() -> Self {
            Struct { a: true }
        }
    }

    #[derive(Encode, Decode, Serializable2)]
    pub struct Struct2 {
        a: bool,
    }

    impl Struct2 {
        pub fn new() -> Self {
            Struct2 { a: true }
        }
    }

    #[test]
    fn test_rust_serializer2() {
        let s = Struct2::new();
        let config = Struct2::config();
        let encoded = s.serialize(&config).unwrap();
        let expected: &[u8] = &[120, 156, 99, 4, 0, 0, 2, 0, 2];
        assert_eq!(expected, encoded);

        let (decoded, _) = Struct2::deserialize(&encoded, &config).unwrap();
        assert_eq!(decoded.a, true);
    }

    #[test]
    fn test_ruby_serializer2() {
        VM::init();
        let ruby_object = RubyObject2::new();
        let encoded = ruby_object.serialize(&()).unwrap();
        let expected: &[u8] = &[120, 156, 99, 225, 48, 0, 0, 0, 79, 0, 61];
        assert_eq!(expected, encoded);

        let (decoded, _) = RubyObject2::deserialize(&encoded, &()).unwrap();
        assert_eq!(decoded.value, ruby_object.value);
    }

    #[test]
    fn test_rust_serializer() {
        let s: Struct = Struct::new();
        let config = Struct::config();
        let encoded = s.encode_to_string(&config).unwrap();
        assert_eq!("AQ==", encoded);

        let (decoded, _) = Struct::decode_from_string(&encoded, &config).unwrap();
        assert_eq!(decoded.a, true);
    }

    #[test]
    fn test_ruby_serializer() {
        VM::init();
        let ruby_object = RubyObject::new();
        let encoded = ruby_object.encode_to_string(&()).unwrap();
        assert_eq!("\u{4}\u{8}0", encoded);

        let (decoded, _) = RubyObject::decode_from_string(&encoded, &()).unwrap();
        assert_eq!(decoded.value, ruby_object.value);
    }
}
