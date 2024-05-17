#[macro_use]
extern crate rutie;
extern crate lazy_static;

use rutie::{AnyObject, Class, Object, RString};

use ccache::serializable::Serializable;
use derive::Serializable;

#[derive(Serializable)]
#[encode_decode(lan = "ruby")]
pub struct RubyObject {
    pub value: rutie::types::Value,
}

pub struct Store {
    inner: ccache::in_memory_store::InMemoryStore<RubyObject>,
    redis_client: redis::Connection,
}

impl Store {
    fn new(redis_url: &str) -> Self {
        Store {
            inner: ccache::in_memory_store::InMemoryStore::new(),
            redis_client: redis::Client::open(redis_url)
                .unwrap()
                .get_connection()
                .unwrap(),
        }
    }
}

wrappable_struct!(Store, StoreWrapper, STORE_WRAPPER);
class!(RubyStore);

methods!(
    RubyStore,
    rtself,
    fn ruby_new(redis_host: RString) -> AnyObject {
        let redis_host = redis_host.unwrap().to_string();
        let store = Store::new(&redis_host);
        Class::from_existing("RubyStore").wrap_data(store, &*STORE_WRAPPER)
    },
    fn ruby_insert(key: RString, obj: AnyObject) -> RString {
        let rbself = rtself.get_data_mut(&*STORE_WRAPPER);
        let ruby_object = RubyObject {
            value: obj.unwrap().value(),
        };

        let etag = rbself
            .inner
            .insert(key.unwrap().to_str(), ruby_object, &mut rbself.redis_client)
            .unwrap();
        RString::new_utf8(&etag.to_string().chars().rev().collect::<String>())
    },
    fn ruby_get(key: RString) -> AnyObject {
        let rbself = rtself.get_data_mut(&*STORE_WRAPPER);
        let object = rbself
            .inner
            .get(key.unwrap().to_str(), &mut rbself.redis_client)
            .unwrap()
            .unwrap();

        AnyObject::from(object.value)
    }
);

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn Init_ruby_example() {
    Class::new("RubyStore", None).define(|klass| {
        klass.def_self("new", ruby_new);
        klass.def("insert", ruby_insert);
        klass.def("get", ruby_get);
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use ccache::in_memory_store::InMemoryStore;
    use rutie::{Boolean, VM};

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
