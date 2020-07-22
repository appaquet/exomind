// create a structure of modules so that generated files can reference exocore's generated code
pub(crate) use exocore::protos as exocore_proto;
pub(crate) mod generated {
    pub(crate) use super::exocore_proto as exocore;
    pub mod exomind {
        pub mod base {
            include!(concat!(env!("OUT_DIR"), "/exomind.base.rs"));
        }
    }
}

pub mod base {
    pub use super::generated::exomind::base::*;
    use exocore::core::protos::message::NamedMessage;

    impl NamedMessage for Collection {
        fn full_name() -> &'static str {
            "exomind.base.Collection"
        }
    }

    impl NamedMessage for CollectionChild {
        fn full_name() -> &'static str {
            "exomind.base.CollectionChild"
        }
    }

    impl NamedMessage for Postponed {
        fn full_name() -> &'static str {
            "exomind.base.Postponed"
        }
    }

    impl NamedMessage for Account {
        fn full_name() -> &'static str {
            "exomind.base.Account"
        }
    }

    impl NamedMessage for EmailThread {
        fn full_name() -> &'static str {
            "exomind.base.EmailThread"
        }
    }

    impl NamedMessage for Email {
        fn full_name() -> &'static str {
            "exomind.base.Email"
        }
    }

    impl NamedMessage for EmailPart {
        fn full_name() -> &'static str {
            "exomind.base.EmailPart"
        }
    }

    impl NamedMessage for EmailAttachment {
        fn full_name() -> &'static str {
            "exomind.base.EmailAttachment"
        }
    }

    impl NamedMessage for Note {
        fn full_name() -> &'static str {
            "exomind.base.Note"
        }
    }

    impl NamedMessage for Contact {
        fn full_name() -> &'static str {
            "exomind.base.Contact"
        }
    }
}
