use std::fmt::Debug;

pub trait Serializable {
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
    extern crate flate2;
    use crate::errors::DecodeError;
    use crate::errors::EncodeError;
    use crate::serializable::Serializable;
    use bincode::{Decode, Encode};
    use derive::Serializable;
    use flate2::Compression;
    use rutie::{NilClass, Object, RString, VM};
    use std::io::Write;
    use rutie::rubysys::string;
    use rutie::types::{c_char, c_long};

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

    #[derive(Encode, Decode, Serializable)]
    pub struct Struct {
        a: bool,
    }

    impl Struct {
        pub fn new() -> Self {
            Struct { a: true }
        }
    }

    #[test]
    fn test_rust_serializer() {
        let s = Struct::new();
        let config = Struct::config();
        let encoded = s.serialize(&config).unwrap();
        let expected: &[u8] = &[120, 156, 99, 4, 0, 0, 2, 0, 2];
        assert_eq!(expected, encoded);

        let (decoded, _) = Struct::deserialize(&encoded, &config).unwrap();
        assert_eq!(decoded.a, true);
    }

    #[test]
    fn test_ruby_serializer() {
        VM::init();
        let ruby_object = RubyObject::new();
        let encoded = ruby_object.serialize(&()).unwrap();
        let expected: &[u8] = &[120, 156, 99, 225, 48, 0, 0, 0, 79, 0, 61];
        assert_eq!(expected, encoded);

        let (decoded, _) = RubyObject::deserialize(&encoded, &()).unwrap();
        assert_eq!(decoded.value, ruby_object.value);
    }
}
