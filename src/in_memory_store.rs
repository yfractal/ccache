use bincode::{config, Decode, Encode};
use redis::Script;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;

pub struct Data<T: Encode + Decode> {
    pub etags: HashMap<String, String>,
    pub structs: HashMap<String, Arc<T>>,
}

impl<T: Encode + Decode> Data<T> {
    pub fn new() -> Self {
        Data {
            structs: HashMap::new(),
            etags: HashMap::new(),
        }
    }
}

pub struct InMemoryStore<T: Encode + Decode> {
    data: RwLock<Data<T>>,
    pub coder_config: config::Configuration,
}

const GET_FROM_REDIS_SCRIPT: &str = r#"
if (redis.call("HGET", KEYS[1], "etag") == ARGV[1]) then
   return {"val","","etag","-1"}
else
   return redis.call("HGETALL", KEYS[1])
end
"#;

const INSERT_TO_REDIS_SCRIPT: &str = r#"
  local time = redis.call('TIME')[1]
  redis.call("HSET", KEYS[1], "val", ARGV[1], "etag", time)

  return time
"#;

impl<T: Encode + Decode> InMemoryStore<T> {
    pub fn new() -> Self {
        Self {
            data: RwLock::new(Data::new()),
            coder_config: config::standard(),
        }
    }

    pub fn insert(
        &self,
        key: &str,
        val: T,
        redis_conn: &mut redis::Connection,
    ) -> Result<String, redis::RedisError> {
        let val_arc = Arc::new(val);
        let etag = self.insert_to_redis(key, val_arc.clone(), redis_conn)?;
        let mut data = self.data.write().unwrap();
        data.etags.insert(key.to_string(), etag.clone());
        data.structs.insert(key.to_string(), val_arc.clone());
        Ok(etag)
    }

    pub fn get(
        &self,
        key: &str,
        redis_conn: &mut redis::Connection,
    ) -> Result<Option<Arc<T>>, redis::RedisError> {
        let data = self.data.read().unwrap();
        match data.etags.get(key) {
            None => {
                let redis_result = get_from_redis_helper(key, redis_conn)?;
                if redis_result.is_empty() {
                    Ok(None) // redis missed
                } else {
                    let val = redis_result.get("val").unwrap();
                    let etag = redis_result.get("etag").unwrap();

                    let (decoded, _): (T, usize) = self.decode_from_string(val).unwrap();

                    let decoded_arc = Arc::new(decoded);
                    // release read lock
                    drop(data);
                    // acquire write lock
                    let mut data = self.data.write().unwrap();
                    data.etags.insert(key.to_string(), etag.to_string());
                    data.structs.insert(key.to_string(), decoded_arc.clone());
                    Ok(Some(decoded_arc))
                }
            }
            Some(etag) => match get_through_local_helper(key, etag, redis_conn) {
                Err(e) => Err(e),
                Ok(GetThroughLocalResult::None) => Ok(None),
                Ok(GetThroughLocalResult::Unchanged) => {
                    let obj = data.structs.get(key).unwrap().clone();
                    Ok(Some(obj.clone()))
                }
                Ok(GetThroughLocalResult::Pair(val, etag)) => {
                    let (decoded, _): (T, usize) = self.decode_from_string(&val).unwrap();
                    let decoded_arc = Arc::new(decoded);
                    drop(data);
                    let mut data = self.data.write().unwrap();
                    data.etags.insert(key.to_string(), etag.to_string());
                    data.structs.insert(key.to_string(), decoded_arc.clone());
                    Ok(Some(decoded_arc))
                }
            },
        }
    }

    fn insert_to_redis<V: Encode>(
        &self,
        key: &str,
        obj: V,
        redis_conn: &mut redis::Connection,
    ) -> Result<String, redis::RedisError> {
        let val = self.encode_obj_to_string(obj).unwrap();
        let etag = self.insert_to_redis_helper(key, val, redis_conn)?;

        Ok(etag.clone())
    }

    fn encode_obj_to_string<E: bincode::enc::Encode>(
        &self,
        val: E,
    ) -> Result<String, bincode::error::EncodeError> {
        let bytes = bincode::encode_to_vec(val, self.coder_config)?;
        Ok(base64::encode(bytes))
    }

    fn insert_to_redis_helper(
        &self,
        key: &str,
        val: String,
        redis_conn: &mut redis::Connection,
    ) -> Result<String, redis::RedisError> {
        let result: Result<String, redis::RedisError> = Script::new(INSERT_TO_REDIS_SCRIPT)
            .key(key)
            .arg(val)
            .invoke(redis_conn);

        result
    }

    fn decode_from_string<D: Decode>(
        &self,
        val: &String,
    ) -> Result<(D, usize), bincode::error::DecodeError> {
        let bytes = base64::decode(val).unwrap();
        bincode::decode_from_slice(&bytes, self.coder_config)
    }
}

fn get_from_redis_helper(
    key: &str,
    conn: &mut redis::Connection,
) -> Result<HashMap<String, String>, redis::RedisError> {
    redis::cmd("HGETALL").arg(key.to_string()).query(conn)
}

enum GetThroughLocalResult {
    None,
    Unchanged,
    Pair(String, String),
}

fn get_through_local_helper(
    key: &str,
    etag: &String,
    conn: &mut redis::Connection,
) -> Result<GetThroughLocalResult, redis::RedisError> {
    let redis_result = get_from_redis_through_etag_helper(key, etag, conn)?;
    if redis_result.is_empty() {
        Ok(GetThroughLocalResult::None)
    } else if redis_result.get("etag").unwrap() == "-1" && redis_result.get("val").unwrap() == "" {
        Ok(GetThroughLocalResult::Unchanged)
    } else {
        let val = redis_result.get("val").unwrap().to_string();
        let etag = redis_result.get("etag").unwrap().to_string();
        Ok(GetThroughLocalResult::Pair(val, etag))
    }
}

fn get_from_redis_through_etag_helper(
    key: &str,
    etag: &String,
    conn: &mut redis::Connection,
) -> Result<HashMap<String, String>, redis::RedisError> {
    Script::new(GET_FROM_REDIS_SCRIPT)
        .key(key)
        .arg(etag.to_string())
        .invoke(conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    impl<T: Encode + Decode> InMemoryStore<T> {
        pub fn delete_etag(&self, key: &str) {
            let mut data = self.data.write().unwrap();
            data.etags.remove(key).unwrap();
        }

        pub fn update_etag(&self, key: &str, etag: &str) {
            let mut data = self.data.write().unwrap();
            *data.etags.get_mut(key).unwrap() = etag.to_string();
        }
    }

    #[derive(Encode, Decode, PartialEq, Debug, Clone)]
    struct Entity {
        x: f32,
        y: f32,
    }

    #[derive(Encode, Decode, PartialEq, Debug, Clone)]
    struct World(Vec<Entity>);

    struct TestContext<T: Encode + Decode> {
        in_memory_store: InMemoryStore<T>,
        redis_conn: redis::Connection,
    }

    impl<T: Encode + Decode> Drop for TestContext<T> {
        fn drop(&mut self) {
            let _: () = redis::cmd("FLUSHDB").query(&mut self.redis_conn).unwrap();
        }
    }

    fn setup<T: Encode + Decode>() -> TestContext<T> {
        let in_memory_store = InMemoryStore::new();

        // Connect to Redis
        let client = redis::Client::open("redis://127.0.0.1/").unwrap();
        let redis_conn = client.get_connection().unwrap();

        TestContext {
            in_memory_store,
            redis_conn,
        }
    }

    #[test]
    fn test_get() {
        let mut ctx = setup();
        let in_memory_store = &mut ctx.in_memory_store;
        let key = "some-key";
        let val = World(vec![Entity { x: 0.0, y: 4.0 }, Entity { x: 10.0, y: 20.5 }]);
        let _ = in_memory_store.insert(key, val, &mut ctx.redis_conn);

        let result = in_memory_store
            .get(key, &mut ctx.redis_conn)
            .unwrap()
            .unwrap();
        assert_eq!(result.0.first().unwrap().x, 0.0);
        assert_eq!(result.0.first().unwrap().y, 4.0);

        assert_eq!(result.0.get(1).unwrap().x, 10.0);
        assert_eq!(result.0.get(1).unwrap().y, 20.5);
    }

    #[test]
    fn test_get_none_exist() {
        let mut ctx = setup::<World>();
        let in_memory_store = &ctx.in_memory_store;
        let result = in_memory_store.get("non-exist-key", &mut ctx.redis_conn);

        assert_eq!(result, Ok(None));
    }

    #[test]
    fn test_get_local_miss_remote_hit() {
        let mut ctx = setup();
        let in_memory_store = &mut ctx.in_memory_store;

        // insert one and delete from local
        in_memory_store
            .insert("some-key", Entity { x: 0.0, y: 4.0 }, &mut ctx.redis_conn)
            .unwrap();
        in_memory_store.delete_etag("some-key");

        let result = in_memory_store
            .get("some-key", &mut ctx.redis_conn)
            .unwrap()
            .unwrap();
        assert_eq!(result.x, 0.0);
        assert_eq!(result.y, 4.0);
    }

    #[test]
    fn test_local_cached_remote_does_not_exist() {
        let mut ctx = setup();
        let in_memory_store = &mut ctx.in_memory_store;

        // insert one and delete redis
        in_memory_store
            .insert("some-key", Entity { x: 0.0, y: 4.0 }, &mut ctx.redis_conn)
            .unwrap();
        redis::cmd("del")
            .arg("some-key".to_string())
            .query::<bool>(&mut ctx.redis_conn)
            .unwrap();

        let result = in_memory_store
            .get("some-key", &mut ctx.redis_conn)
            .unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_local_miss() {
        let mut ctx = setup();
        let in_memory_store = &mut ctx.in_memory_store;

        // insert one and update etag
        in_memory_store
            .insert("some-key", Entity { x: 0.0, y: 4.0 }, &mut ctx.redis_conn)
            .unwrap();
        in_memory_store.update_etag("some-key", "abc");

        let result = in_memory_store
            .get("some-key", &mut ctx.redis_conn)
            .unwrap()
            .unwrap();
        assert_eq!(result.x, 0.0);
        assert_eq!(result.y, 4.0);
    }
}
