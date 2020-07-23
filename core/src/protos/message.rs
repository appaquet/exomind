use super::test::{TestMessage, TestMessage2};

pub trait NamedMessage {
    fn full_name() -> &'static str;
}

impl NamedMessage for TestMessage {
    fn full_name() -> &'static str {
        "exocore.test.TestMessage"
    }
}

impl NamedMessage for TestMessage2 {
    fn full_name() -> &'static str {
        "exocore.test.TestMessage2"
    }
}
