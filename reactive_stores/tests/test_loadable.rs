use reactive_stores::{Loadable, Store};
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, Default)]
struct TestError;

impl fmt::Display for TestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TestError")
    }
}

impl Error for TestError {}

#[derive(Debug, Clone, Default)]
struct Item {
    pub id: i32,
    pub name: String,
}

#[derive(Store, Debug, Default)]
struct TestState {
    pub counter: i32,
    #[store(loadable)]
    pub simple_loading: Loadable<(), TestError>,
    #[store(loadable, key: i32 = |item| item.id)]
    pub keyed_loading: Loadable<Vec<Item>, TestError>,
}

#[test]
fn test_with_loadable_and_keyed() {
    let _ = TestState::default();
}
