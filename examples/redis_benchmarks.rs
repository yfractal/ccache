use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use std::env;
use std::thread;

fn generate_random_string(length: usize) -> String {
    let rng = thread_rng();
    let random_string: String = rng
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect();
    random_string
}

fn inject_data(
    total_count: usize,
    val_size: usize,
    redis_conn: &mut redis::Connection,
) -> Vec<String> {
    let keys = (0..total_count)
        .map(|i| generate_random_string(5) + &i.to_string())
        .collect();
    for key in &keys {
        let val = generate_random_string(val_size);
        let _: () = redis::cmd("SET")
            .arg(key)
            .arg(val)
            .query(redis_conn)
            .unwrap();
    }
    keys
}

fn get_from_redis(key: &str, redis_conn: &mut redis::Connection) -> String {
    redis::cmd("GET").arg(key).query(redis_conn).unwrap()
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let keys_count = args[1].parse::<usize>().unwrap();
    let val_size = args[2].parse::<usize>().unwrap(); // in bytes
    let threads_count = args[3].parse::<usize>().unwrap();
    print!(
        "input: keys_count={}, val_size={}, threads_count={}",
        keys_count, val_size, threads_count
    );

    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let mut redis_conn = client.get_connection().unwrap();
    let keys = inject_data(keys_count, val_size, &mut redis_conn);

    let handles: Vec<_> = (0..threads_count)
        .map(|_| {
            let keys = keys.clone();
            let client = redis::Client::open("redis://127.0.0.1/").unwrap();
            let mut redis_conn = client.get_connection().unwrap();
            thread::spawn(move || {
                let mut rng = rand::thread_rng();
                loop {
                    let random_number: usize = rng.gen_range(0..keys_count);
                    let key = keys.get(random_number).unwrap();
                    get_from_redis(key, &mut redis_conn);
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }
}
