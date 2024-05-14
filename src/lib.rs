pub mod in_memory_store;
pub mod serialize;

// use derive::MyTrait;
use derive::EncodeDecode;
use bincode::{config, Decode, Encode};

trait EncodeDecode {
    type Error;
    type DecodeError;
    type Config;

    fn encode_to_string(&self, config: Self::Config) -> Result<String, Self::Error>;
    fn decode_from_string(val: &String, config: Self::Config) -> Result<(Self, usize), Self::DecodeError> where Self: Sized;
}

#[derive(Encode, Decode, EncodeDecode)]
pub struct Store2{
    a: bool
}

impl Store2 {
    pub fn new() -> Self {
        Store2 {
            a: true,
        }
    }
}

#[test]
fn default() {
    let store: Store2  = Store2::new();
    let config = config::standard();
    let encoded =  store.encode_to_string(config).unwrap();
    assert_eq!("AQ==", encoded);

    let (decoded, _) = Store2::decode_from_string(&encoded, config).unwrap();
    assert_eq!(decoded.a, true);
}
