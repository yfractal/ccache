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

#[cfg(test)]
mod serializer_tests {
    use crate::serializer::Serializable;
    use bincode::{Decode, Encode};
    use derive::Serializable;
    use rutie::{AnyObject, Class, Marshal, NilClass, Object, RString, VM};

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
        let s: Struct = Struct::new();
        let config = Struct::config();
        let encoded = s.encode_to_string(&config).unwrap();
        assert_eq!("AQ==", encoded);

        let (decoded, _) = Struct::decode_from_string(&encoded, &config).unwrap();
        assert_eq!(decoded.a, true);
    }

    #[test]
    fn test_ruby_serializer() {
        let s: Struct = Struct::new();
        let config = Struct::config();
        let encoded = s.encode_to_string(&config).unwrap();
        assert_eq!("AQ==", encoded);

        let (decoded, _) = Struct::decode_from_string(&encoded, &config).unwrap();
        assert_eq!(decoded.a, true);
    }
}
