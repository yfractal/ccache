use derive::Serializable;
use nimbus::in_memory_store::InMemoryStore;
use nimbus::serializer::Serializable;
use rutie::{Boolean, Object, RString, VM};

#[derive(Serializable)]
#[encode_decode(lan = "ruby")]
pub struct RubyObject {
    pub value: rutie::types::Value,
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        VM::init();
        let store = InMemoryStore::new();
        let client = redis::Client::open("redis://127.0.0.1/").unwrap();
        let mut redis_conn = client.get_connection().unwrap();
        let ruby_object = RubyObject {
            value: Boolean::new(true).value(),
        };
        store.insert("a-key", ruby_object, &mut redis_conn).unwrap();
        let inserted = store.get("a-key", &mut redis_conn).unwrap();

        assert_eq!(true, inserted.unwrap().value == Boolean::new(true).value());
    }
}
