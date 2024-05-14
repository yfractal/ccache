pub trait Serializer {
    type Error;
    type DecodeError;
    type Config;

    fn encode_to_string(&self, config: Self::Config) -> Result<String, Self::Error>;
    fn decode_from_string(
        val: &String,
        config: Self::Config,
    ) -> Result<(Self, usize), Self::DecodeError>
    where
        Self: Sized;
}

#[cfg(test)]
mod serializer_tests {
    use crate::serializer::Serializer;
    use bincode::{config, Decode, Encode};
    use derive::Serializer;

    #[derive(Encode, Decode, Serializer)]
    #[encode_decode(lan = "ruby")]
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
        let config = config::standard();
        let encoded = s.encode_to_string(config).unwrap();
        assert_eq!("AQ==", encoded);

        let (decoded, _) = Struct::decode_from_string(&encoded, config).unwrap();
        assert_eq!(decoded.a, true);
    }
}
