pub mod in_memory_store;
pub mod serialize;

// use derive::MyTrait;
use derive::EncodeDecode;
use bincode::{config, Decode, Encode};

trait EncodeDecode {
    type Error;
    type Config;

    fn encode_to_string(&self, config: Self::Config) -> Result<String, Self::Error>;
    fn decode_from_string(val: &String, config: Self::Config) -> Result<String, Self::Error> where Self: Sized;
}

// expect
// #[derive(EncodeDecode)]
// #[ruby]
// struct A {}
// it has ruby implemention

//  #[rust]
// struct A {}
// it has rust implemention


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
    print!("encoded {:?}", encoded);

    // assert_eq!(Foo::answer(), 42);
    // let store = Store{}

    // let store: Store = Store{coder_config: ()};
    // assert_eq!(store.encode_to_string(&store.coder_config).unwrap(), "test".to_string());
    // assert_eq!(Store::decode_from_string(&"".to_string(), &store.coder_config).unwrap(), "abc".to_string());
}
