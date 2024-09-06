// create a structure of modules so that generated files can reference exocore's generated code
pub(crate) use exocore::protos as exocore_proto;
pub(crate) mod generated {
    pub(crate) use super::exocore_proto as exocore;
    pub mod exomind {
        pub mod v1 {
            #[path = "../../exomind.base.v1.rs"]
            pub mod base;
        }
    }
}
pub mod base {
    pub use super::generated::exomind::v1::base::*;
    use exocore::protos::message::NamedMessage;

    impl NamedMessage for Collection {
        fn full_name() -> &'static str {
            "exomind.base.v1.Collection"
        }
    }

    impl NamedMessage for CollectionChild {
        fn full_name() -> &'static str {
            "exomind.base.v1.CollectionChild"
        }
    }

    impl NamedMessage for Snoozed {
        fn full_name() -> &'static str {
            "exomind.base.v1.Snoozed"
        }
    }

    impl NamedMessage for Unread {
        fn full_name() -> &'static str {
            "exomind.base.v1.Unread"
        }
    }

    impl NamedMessage for Account {
        fn full_name() -> &'static str {
            "exomind.base.v1.Account"
        }
    }

    impl NamedMessage for EmailThread {
        fn full_name() -> &'static str {
            "exomind.base.v1.EmailThread"
        }
    }

    impl NamedMessage for Email {
        fn full_name() -> &'static str {
            "exomind.base.v1.Email"
        }
    }

    impl NamedMessage for DraftEmail {
        fn full_name() -> &'static str {
            "exomind.base.v1.DraftEmail"
        }
    }

    impl NamedMessage for EmailPart {
        fn full_name() -> &'static str {
            "exomind.base.v1.EmailPart"
        }
    }

    impl NamedMessage for EmailAttachment {
        fn full_name() -> &'static str {
            "exomind.base.v1.EmailAttachment"
        }
    }

    impl NamedMessage for Note {
        fn full_name() -> &'static str {
            "exomind.base.v1.Note"
        }
    }

    impl NamedMessage for Task {
        fn full_name() -> &'static str {
            "exomind.base.v1.Task"
        }
    }

    impl NamedMessage for Link {
        fn full_name() -> &'static str {
            "exomind.base.v1.Link"
        }
    }

    impl NamedMessage for Contact {
        fn full_name() -> &'static str {
            "exomind.base.v1.Contact"
        }
    }
}
