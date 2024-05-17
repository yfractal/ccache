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
    use crate::serializable::Serializable;
    use bincode::{Decode, Encode};
    use derive::Serializable;
    use rutie::{NilClass, Object, RString, VM};

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
        VM::init();
        let ruby_object = RubyObject::new();
        let encoded = ruby_object.encode_to_string(&()).unwrap();
        assert_eq!("\u{4}\u{8}0", encoded);

        let (decoded, _) = RubyObject::decode_from_string(&encoded, &()).unwrap();
        assert_eq!(decoded.value, ruby_object.value);
    }
}