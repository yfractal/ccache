use bincode::{Decode, Encode};
use ccache::in_memory_store::InMemoryStore;
use ccache::serializable::Serializable;
use derive::Serializable;
use std::sync::{Arc, Mutex};
use tide::Request;

#[derive(Encode, Decode, Serializable, PartialEq, Debug, Clone)]
struct Entity {
    x: f32,
    y: f32,
}

#[derive(Encode, Decode, Serializable, PartialEq, Debug, Clone)]
struct World(Vec<Entity>);

struct AppState<T: Serializable> {
    in_memory_store: Arc<InMemoryStore<T>>,
    redis_conn: Arc<Mutex<redis::Connection>>,
}

impl<T: Serializable> AppState<T> {
    pub fn new() -> Self {
        let client = redis::Client::open("redis://127.0.0.1/").unwrap();
        let redis_conn = client.get_connection().unwrap();
        Self {
            in_memory_store: Arc::new(InMemoryStore::<T>::new()),
            redis_conn: Arc::new(Mutex::new(redis_conn)),
        }
    }
}

async fn handle_get(req: Request<Arc<AppState<World>>>) -> tide::Result {
    let state = req.state().clone();
    let store = state.in_memory_store.clone();
    let rv = store.get("some-key", &mut state.redis_conn.clone().lock().unwrap());
    println!("w=#{:?}", rv);

    Ok(format!("Hello").into())
}

async fn handle_post(req: Request<Arc<AppState<World>>>) -> tide::Result {
    let state = req.state().clone();
    let store = state.in_memory_store.clone();
    let w = World(vec![Entity { x: 0.0, y: 4.0 }, Entity { x: 10.0, y: 20.5 }]);
    store
        .insert("some-key", w, &mut state.redis_conn.clone().lock().unwrap())
        .unwrap();

    Ok(format!("Hello post").into())
}

#[async_std::main]
async fn main() -> tide::Result<()> {
    let app_state: Arc<AppState<World>> = Arc::new(AppState::new());
    let mut app = tide::with_state(app_state);

    app.at("/").get(handle_get);
    app.at("/").post(handle_post);
    app.listen("127.0.0.1:8080").await?;
    Ok(())
}
