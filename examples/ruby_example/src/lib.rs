#[macro_use]
extern crate rutie;
extern crate flate2;
extern crate lazy_static;

use ccache::errors::DecodeError;
use ccache::errors::EncodeError;
use ccache::serializable::Serializable;
use derive::Serializable;
use flate2::Compression;
use rutie::rubysys::string;
use rutie::types::{c_char, c_long};
use rutie::{AnyObject, Class, NilClass, Object, RString, VM};
use std::io::Write;

#[derive(Serializable, Debug)]
#[encode_decode(lan = "ruby")]
pub struct RubyObject {
    pub value: rutie::types::Value,
}

pub struct Store {
    inner: ccache::in_memory_store::InMemoryStore<RubyObject>,
    redis_client: redis::Connection,
}

impl Store {
    fn new(redis_url: &str) -> Result<Self, redis::RedisError> {
        let redis_client = redis::Client::open(redis_url)?;
        let redic_connection = redis_client.get_connection()?;

        let store = Store {
            inner: ccache::in_memory_store::InMemoryStore::new(),
            redis_client: redic_connection,
        };

        Ok(store)
    }
}

wrappable_struct!(Store, StoreWrapper, STORE_WRAPPER);
class!(RubyStore);

methods!(
    RubyStore,
    rtself,
    fn ruby_new(redis_host: RString) -> AnyObject {
        let redis_host = redis_host.unwrap().to_string();

        match Store::new(&redis_host) {
            Ok(store) => Class::from_existing("RubyStore").wrap_data(store, &*STORE_WRAPPER),
            Err(error) => {
                let error_class = Class::from_existing("CcacheRedisError");
                VM::raise(error_class, &error.to_string());
                NilClass::new().into()
            }
        }
    },
    fn ruby_insert(key: RString, obj: AnyObject) -> AnyObject {
        let rbself = rtself.get_data_mut(&*STORE_WRAPPER);
        let ruby_object = RubyObject {
            value: obj.unwrap().value(),
        };

        match rbself
            .inner
            .insert(key.unwrap().to_str(), ruby_object, &mut rbself.redis_client)
        {
            Ok(etag) => RString::new_utf8(&String::from_utf8(etag).unwrap()).into(),
            Err(error) => {
                let error_class = Class::from_existing("CcacheRedisError");
                VM::raise(error_class, &error.to_string());
                NilClass::new().into()
            }
        }
    },
    fn rs_get(key: RString) -> AnyObject {
        let rbself = rtself.get_data_mut(&*STORE_WRAPPER);
        let result = rbself
            .inner
            .get(key.unwrap().to_str(), &mut rbself.redis_client);

        match result {
            Ok(Some(val)) => AnyObject::from(val.value),
            Ok(None) => NilClass::new().into(),
            Err(error) => {
                let error_class = Class::from_existing("CcacheRedisError");
                VM::raise(error_class, &error.to_string());
                NilClass::new().into()
            }
        }
    }
);

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn Init_ruby_example() {
    Class::new("RubyStore", None).define(|klass| {
        klass.def_self("new", ruby_new);
        klass.def("insert", ruby_insert);
        klass.def_private("rs_get", rs_get);
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use ccache::in_memory_store::InMemoryStore;
    use rutie::{AnyException, Class, Exception, Object};
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
