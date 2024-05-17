#[macro_use]
extern crate rutie;

use rutie::{Class, Boolean, Object, RString, VM};

use derive::Serializable;
use nimbus::in_memory_store::InMemoryStore;
use nimbus::serializer::Serializable;

#[derive(Serializable)]
#[encode_decode(lan = "ruby")]
pub struct RubyObject {
    pub value: rutie::types::Value,
}

class!(RutieExample);

methods!(
    RutieExample,
    _rtself,

    fn pub_reverse(input: RString) -> RString {
        let ruby_string = input.
          map_err(|e| VM::raise_ex(e) ).
          unwrap();

        RString::new_utf8(
          &ruby_string.
          to_string().
          chars().
          rev().
          collect::<String>()
        )
    }
);

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn Init_ruby_example() {
    // RutieExample.reverse "abc"
    Class::new("RutieExample", None).define(|klass| {
        klass.def_self("reverse", pub_reverse);
    });
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
