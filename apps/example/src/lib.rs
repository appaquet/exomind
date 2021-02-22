//! Sample application used for integration tests in application runtime. Don't
//! modify it without changing tests.

#[macro_use]
extern crate log;

use std::time::Duration;

use exocore::{
    apps::sdk::prelude::*,
    store::{mutation::MutationBuilder, query::QueryBuilder},
};

#[exocore_app]
pub struct TestApplication {}

impl TestApplication {
    fn new() -> Self {
        TestApplication {}
    }
}

impl App for TestApplication {
    fn start(&self, exocore: &Exocore) -> Result<(), AppError> {
        info!("application initialized");

        let store = exocore.store.clone();
        spawn(async move {
            info!("task starting");

            info!("before sleep {}", now().0);
            sleep(Duration::from_millis(100)).await;
            info!("after sleep {}", now().0);

            info!("before mutation");
            let m = MutationBuilder::new().delete_entity("entity1").build();
            match store.mutate(m).await {
                Ok(_) => info!("mutation success"),
                Err(err) => info!("mutation error: {}", err),
            }

            info!("before query");
            let q = QueryBuilder::with_id("test").build();
            match store.query(q).await {
                Ok(_) => info!("query success"),
                Err(err) => info!("query error: {}", err),
            }

            info!("task done")
        });

        Ok(())
    }
}
