use derive::MyTrait;

pub mod in_memory_store;

trait MyTrait {
    fn answer() -> i32 {
        42
    }
}

#[derive(MyTrait)]
struct Foo;

#[test]
fn default() {
    assert_eq!(Foo::answer(), 42);
}
