pub mod in_memory_store;
pub mod serialize;

use derive::MyTrait;
use derive::EncodeDecode;

trait MyTrait {
    fn answer() -> i32 {
        42
    }
}

#[derive(MyTrait)]
struct Foo;

trait EncodeDecode {
    type Error;
    type Config;

    fn encode_to_string(&self, config: &Self::Config) -> Result<String, Self::Error>;
    fn decode_from_string(val: &String, config: &Self::Config) -> Result<String, Self::Error> where Self: Sized;
}

#[derive(EncodeDecode)]
pub struct Store {
    pub coder_config: (),
}

#[test]
fn default() {
    assert_eq!(Foo::answer(), 42);

    let store: Store = Store{coder_config: ()};
    assert_eq!(store.encode_to_string(&store.coder_config).unwrap(), "test".to_string());
    assert_eq!(Store::decode_from_string(&"".to_string(), &store.coder_config).unwrap(), "abc".to_string());
}
