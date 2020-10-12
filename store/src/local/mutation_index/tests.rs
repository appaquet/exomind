use chrono::{DateTime, Utc};
use itertools::Itertools;

use exocore_chain::operation::OperationId;
use exocore_core::protos::generated::exocore_store::{Reference, Trait};
use exocore_core::protos::generated::exocore_test::{TestMessage, TestMessage2};
use exocore_core::protos::{
    prost::{Any, ProstAnyPackMessageExt, ProstDateTimeExt},
    store::TraitDetails,
};

use crate::ordering::{value_from_f32, value_from_u64, OrderingValueExt};
use crate::query::{QueryBuilder as Q, TraitQueryBuilder as TQ};

use super::*;

#[test]
fn fetch_entity_mutations() -> anyhow::Result<()> {
    let registry = Arc::new(Registry::new_with_exocore_types());
    let config = test_config();
    let mut index = MutationIndex::create_in_memory(config, registry)?;

    let trait1 = IndexOperation::PutTrait(PutTraitMutation {
        block_offset: Some(1),
        operation_id: 10,
        entity_id: "entity_id1".to_string(),
        trt: Trait {
            id: "foo1".to_string(),
            message: Some(
                TestMessage {
                    string1: "Foo Foo Foo Foo Bar".to_string(),
                    ..Default::default()
                }
                .pack_to_any()?,
            ),
            ..Default::default()
        },
    });

    let trait2 = IndexOperation::PutTrait(PutTraitMutation {
        block_offset: Some(2),
        operation_id: 20,
        entity_id: "entity_id2".to_string(),
        trt: Trait {
            id: "foo2".to_string(),
            message: Some(
                TestMessage {
                    string1: "Foo Foo Foo Foo Bar".to_string(),
                    ..Default::default()
                }
                .pack_to_any()?,
            ),
            ..Default::default()
        },
    });

    let trait3 = IndexOperation::PutTrait(PutTraitMutation {
        block_offset: Some(3),
        operation_id: 21,
        entity_id: "entity_id2".to_string(),
        trt: Trait {
            id: "foo3".to_string(),
            message: Some(
                TestMessage {
                    string1: "Foo Foo Foo Foo Bar".to_string(),
                    ..Default::default()
                }
                .pack_to_any()?,
            ),
            ..Default::default()
        },
    });
    index.apply_operations(vec![trait1, trait2, trait3].into_iter())?;

    let query = Q::with_id("entity_id1").build();
    let res = index.search(query)?;
    assert_eq!(res.mutations.len(), 1);
    assert_eq!(res.mutations[0].block_offset, Some(1));
    assert_eq!(res.mutations[0].operation_id, 10);
    assert_eq!(res.mutations[0].entity_id.as_str(), "entity_id1");
    assert_is_put_trait(&res.mutations[0].mutation_type, "foo1");

    let query = Q::with_id("entity_id2").build();
    let res = index.search(query)?;
    assert_eq!(res.mutations.len(), 2);
    find_put_trait(&res, "foo2");
    find_put_trait(&res, "foo3");

    // search all should return an iterator all results
    let query = Q::with_id("entity_id2").build();
    let iter = index.search_iter(query)?;
    assert_eq!(iter.total_results, 2);
    let results = iter.collect_vec();
    assert_eq!(results.len(), 2);

    Ok(())
}

#[test]
fn search_query_matches() -> anyhow::Result<()> {
    let registry = Arc::new(Registry::new_with_exocore_types());
    let config = test_config();
    let mut index = MutationIndex::create_in_memory(config, registry)?;

    let trait1 = IndexOperation::PutTrait(PutTraitMutation {
        block_offset: Some(1),
        operation_id: 10,
        entity_id: "entity_id1".to_string(),
        trt: Trait {
            id: "foo1".to_string(),
            message: Some(
                TestMessage {
                    string1: "Foo Foo Foo Foo Bar".to_string(),
                    ..Default::default()
                }
                .pack_to_any()?,
            ),
            ..Default::default()
        },
    });

    let trait2 = IndexOperation::PutTrait(PutTraitMutation {
        block_offset: Some(2),
        operation_id: 20,
        entity_id: "entity_id2".to_string(),
        trt: Trait {
            id: "foo2".to_string(),
            message: Some(
                TestMessage {
                    string1: "Foo Bar Bar Bar Bar".to_string(),
                    ..Default::default()
                }
                .pack_to_any()?,
            ),
            ..Default::default()
        },
    });
    index.apply_operations(vec![trait1, trait2].into_iter())?;

    let query = Q::matches("foo").build();
    let res = index.search(query)?;
    assert_eq!(res.mutations.len(), 2);
    assert_eq!(res.mutations[0].entity_id.as_str(), "entity_id1"); // foo is repeated in entity 1

    let query = Q::matches("bar").build();
    let res = index.search(query)?;
    assert_eq!(res.mutations.len(), 2);
    assert!(res.mutations[0]
        .sort_value
        .value
        .is_after(&value_from_f32(0.30, 0)));
    assert!(res.mutations[1]
        .sort_value
        .value
        .is_after(&value_from_f32(0.18, 0)));
    assert_eq!(res.mutations[0].entity_id.as_str(), "entity_id2"); // foo is repeated in entity 2

    // with limit
    let query = Q::matches("foo").count(1).build();
    let res = index.search(query)?;
    assert_eq!(res.mutations.len(), 1);
    assert_eq!(res.remaining, 1);
    assert_eq!(res.total, 2);

    // only results from given score
    let paging = Paging {
        after_ordering_value: Some(value_from_f32(0.30, 0)),
        before_ordering_value: None,
        count: 10,
    };
    let query = Q::matches("bar").with_paging(paging).build();
    let res = index.search(query)?;
    assert_eq!(res.mutations.len(), 1);
    assert_eq!(res.remaining, 0);
    assert_eq!(res.total, 2);
    assert_eq!(res.mutations[0].entity_id.as_str(), "entity_id2");

    // only results before given score
    let paging = Paging {
        after_ordering_value: None,
        before_ordering_value: Some(value_from_f32(0.30, 0)),
        count: 10,
    };
    let query = Q::matches("bar").with_paging(paging).build();
    let res = index.search(query)?;
    assert_eq!(res.mutations.len(), 1);
    assert_eq!(res.remaining, 0);
    assert_eq!(res.total, 2);
    assert_eq!(res.mutations[0].entity_id.as_str(), "entity_id1");

    Ok(())
}

#[test]
fn search_query_matches_paging() -> anyhow::Result<()> {
    let registry = Arc::new(Registry::new_with_exocore_types());
    let config = test_config();
    let mut index = MutationIndex::create_in_memory(config, registry)?;

    let traits = (0..30).map(|i| {
        let text = "foo ".repeat((i + 1) as usize);

        IndexOperation::PutTrait(PutTraitMutation {
            block_offset: Some(i),
            operation_id: i,
            entity_id: format!("entity_id{}", i),
            trt: Trait {
                id: format!("id{}", i),
                message: Some(
                    TestMessage {
                        string1: text,
                        ..Default::default()
                    }
                    .pack_to_any()
                    .unwrap(),
                ),
                ..Default::default()
            },
        })
    });
    index.apply_operations(traits)?;

    let query = Q::matches("foo").count(10).build();
    let res1 = index.search(query)?;
    let res1_next_page = res1.next_page.clone().unwrap();
    assert_eq!(res1.total, 30);
    assert_eq!(res1.mutations.len(), 10);
    assert_eq!(res1.remaining, 20);
    let ids = extract_traits_id(&res1);
    assert_eq!(ids[0], "id29");
    assert_eq!(ids[9], "id20");

    let query = Q::matches("foo").with_paging(res1_next_page).build();
    let res2 = index.search(query)?;
    let res2_next_page = res2.next_page.clone().unwrap();
    assert_eq!(res2.total, 30);
    assert_eq!(res2.mutations.len(), 10);
    assert_eq!(res2.remaining, 10);
    let ids = extract_traits_id(&res2);
    assert_eq!(ids[0], "id19");
    assert_eq!(ids[9], "id10");

    let query = Q::matches("foo").with_paging(res2_next_page).build();
    let res3 = index.search(query)?;
    assert_eq!(res3.total, 30);
    assert_eq!(res3.mutations.len(), 10);
    assert_eq!(res3.remaining, 0);
    let ids = extract_traits_id(&res3);
    assert_eq!(ids[0], "id9");
    assert_eq!(ids[9], "id0");

    // search all should return an iterator over all results
    let query = Q::matches("foo").build();
    let iter = index.search_iter(query)?;
    assert_eq!(iter.total_results, 30);
    let ops: Vec<OperationId> = iter.map(|r| r.operation_id).collect();
    assert_eq!(ops.len(), 30);
    assert_eq!(ops[0], 29);
    assert_eq!(ops[29], 0);

    // reversed order
    let query = Q::matches("foo").order_ascending(true).build();
    let iter = index.search_iter(query)?;
    assert_eq!(iter.total_results, 30);
    let ops: Vec<OperationId> = iter.map(|r| r.operation_id).collect();
    assert_eq!(ops.len(), 30);
    assert_eq!(ops[0], 0);
    assert_eq!(ops[29], 29);

    Ok(())
}

#[test]
fn search_query_by_trait_type() -> anyhow::Result<()> {
    let registry = Arc::new(Registry::new_with_exocore_types());
    let config = test_config();
    let mut index = MutationIndex::create_in_memory(config, registry)?;

    let trait1 = IndexOperation::PutTrait(PutTraitMutation {
        block_offset: None,
        operation_id: 1,
        entity_id: "entity_id1".to_string(),
        trt: Trait {
            id: "trait1".to_string(),
            message: Some(
                TestMessage {
                    string1: "Foo Bar".to_string(),
                    ..Default::default()
                }
                .pack_to_any()?,
            ),
            ..Default::default()
        },
    });

    let trait2 = IndexOperation::PutTrait(PutTraitMutation {
        block_offset: None,
        operation_id: 2,
        entity_id: "entity_id2".to_string(),
        trt: Trait {
            id: "trait2".to_string(),
            message: Some(
                TestMessage2 {
                    string1: "Some subject".to_string(),
                    ..Default::default()
                }
                .pack_to_any()?,
            ),
            ..Default::default()
        },
    });

    let trait3 = IndexOperation::PutTrait(PutTraitMutation {
        block_offset: None,
        operation_id: 3,
        entity_id: "entity_id3".to_string(),
        trt: Trait {
            id: "trait3".to_string(),
            message: Some(
                TestMessage2 {
                    string1: "Some subject".to_string(),
                    ..Default::default()
                }
                .pack_to_any()?,
            ),
            ..Default::default()
        },
    });

    let trait4 = IndexOperation::PutTrait(PutTraitMutation {
        block_offset: None,
        operation_id: 4,
        entity_id: "entity_id4".to_string(),
        trt: Trait {
            id: "trait4".to_string(),
            message: Some(
                TestMessage2 {
                    string1: "Some subject".to_string(),
                    ..Default::default()
                }
                .pack_to_any()?,
            ),
            ..Default::default()
        },
    });

    index.apply_operations(vec![trait4, trait3, trait2, trait1].into_iter())?;

    let query = Q::with_trait::<TestMessage>().build();
    let res = index.search(query)?;
    assert_eq!(res.mutations.len(), 1);
    let trait1 = find_put_trait(&res, "trait1").unwrap();
    match &trait1.mutation_type {
        MutationType::TraitPut(pt) => {
            assert_eq!(pt.trait_type.as_deref(), Some("exocore.test.TestMessage"))
        }
        other => {
            panic!("Expected trait put mutation, got {:?}", other);
        }
    }

    // ordering of multiple traits is operation id
    let query = Q::with_trait::<TestMessage2>().build();
    let res = index.search(query)?;
    assert_eq!(extract_traits_id(&res), vec!["trait4", "trait3", "trait2"]);
    let trait4 = find_put_trait(&res, "trait4").unwrap();
    match &trait4.mutation_type {
        MutationType::TraitPut(pt) => {
            assert_eq!(pt.trait_type.as_deref(), Some("exocore.test.TestMessage2"))
        }
        other => {
            panic!("Expected trait put mutation, got {:?}", other);
        }
    }

    // with limit
    let query = Q::with_trait::<TestMessage2>().count(1).build();
    let res = index.search(query)?;
    assert_eq!(res.mutations.len(), 1);

    // only results after given modification date
    let paging = Paging {
        after_ordering_value: Some(value_from_u64(2, u64::max_value())),
        before_ordering_value: None,
        count: 10,
    };
    let query = Q::with_trait::<TestMessage2>().with_paging(paging).build();
    let res = index.search(query)?;
    assert_eq!(extract_traits_id(&res), vec!["trait4", "trait3"]);

    // only results before given modification date
    let paging = Paging {
        after_ordering_value: None,
        before_ordering_value: Some(value_from_u64(3, 0)),
        count: 10,
    };
    let query = Q::with_trait::<TestMessage2>().with_paging(paging).build();
    let res = index.search(query)?;
    assert_eq!(extract_traits_id(&res), vec!["trait2"]);

    Ok(())
}

#[test]
fn search_query_by_trait_type_paging() -> anyhow::Result<()> {
    let registry = Arc::new(Registry::new_with_exocore_types());
    let config = test_config();
    let mut index = MutationIndex::create_in_memory(config, registry)?;

    let traits = (0..30).map(|i| {
        IndexOperation::PutTrait(PutTraitMutation {
            block_offset: Some(i),
            operation_id: 30 - i,
            entity_id: format!("entity_id{}", i),
            trt: Trait {
                id: format!("entity_id{}", i),
                message: Some(
                    TestMessage {
                        string1: "Some Subject".to_string(),
                        ..Default::default()
                    }
                    .pack_to_any()
                    .unwrap(),
                ),
                ..Default::default()
            },
        })
    });
    index.apply_operations(traits)?;

    let query = Q::with_trait::<TestMessage>().count(10).build();
    let res1 = index.search(query)?;
    assert_eq!(res1.total, 30);
    assert_eq!(res1.remaining, 20);
    assert_eq!(res1.mutations.len(), 10);
    find_put_trait(&res1, "id29");
    find_put_trait(&res1, "id20");

    let query = Q::with_trait::<TestMessage>()
        .with_paging(res1.next_page.unwrap())
        .build();
    let res2 = index.search(query)?;
    assert_eq!(res2.total, 30);
    assert_eq!(res2.remaining, 10);
    assert_eq!(res2.mutations.len(), 10);
    find_put_trait(&res2, "id19");
    find_put_trait(&res2, "id10");

    let query = Q::with_trait::<TestMessage>()
        .with_paging(res2.next_page.unwrap())
        .build();
    let res3 = index.search(query)?;
    assert_eq!(res3.total, 30);
    assert_eq!(res3.remaining, 0);
    assert_eq!(res3.mutations.len(), 10);
    find_put_trait(&res3, "id9");
    find_put_trait(&res3, "id0");

    // search all should return an iterator over all results
    let query = Q::with_trait::<TestMessage>().build();
    let iter = index.search_iter(query)?;
    assert_eq!(iter.total_results, 30);
    let results = iter.collect_vec();
    assert_eq!(results.len(), 30);

    Ok(())
}

#[test]
fn sort_by_field() -> anyhow::Result<()> {
    let registry = Arc::new(Registry::new_with_exocore_types());
    let config = test_config();
    let mut index = MutationIndex::create_in_memory(config, registry)?;

    let traits = (0..20).map(|i| {
        IndexOperation::PutTrait(PutTraitMutation {
            block_offset: Some(i),
            operation_id: 20 - i,
            entity_id: format!("entity_id{}", i),
            trt: Trait {
                id: format!("trait{}", i),
                message: Some(
                    TestMessage {
                        string1: "Some Subject".to_string(),
                        uint3: i as u32,
                        ..Default::default()
                    }
                    .pack_to_any()
                    .unwrap(),
                ),
                ..Default::default()
            },
        })
    });
    index.apply_operations(traits)?;

    let q1 = Q::with_trait_query::<TestMessage>(TQ::matches("subject").build())
        .order_by_field("uint3", false)
        .count(10);
    let res1 = index.search(&q1.clone().build())?;
    let trt1 = extract_traits_id(&res1);
    assert_eq!(trt1[0], "trait19");
    assert_eq!(trt1[9], "trait10");

    let q2 = q1.clone().with_paging(res1.next_page.unwrap());
    let res2 = index.search(&q2.build())?;
    let trt2 = extract_traits_id(&res2);
    assert_eq!(trt2[0], "trait9");
    assert_eq!(trt2[9], "trait0");

    let q3 = q1.count(20).order_ascending(true);
    let res3 = index.search(&q3.build())?;
    let trt3 = extract_traits_id(&res3);
    assert_eq!(trt3[0], "trait0");
    assert_eq!(trt3[19], "trait19");

    Ok(())
}

#[test]
fn search_by_reference() -> anyhow::Result<()> {
    let registry = Arc::new(Registry::new_with_exocore_types());
    let config = test_config();
    let mut index = MutationIndex::create_in_memory(config, registry)?;

    let et1 = IndexOperation::PutTrait(PutTraitMutation {
        block_offset: None,
        operation_id: 1,
        entity_id: "et1".to_string(),
        trt: Trait {
            id: "trt1".to_string(),
            message: Some(
                TestMessage {
                    string1: "Foo Bar".to_string(),
                    ref1: Some(Reference {
                        entity_id: "et2".to_string(),
                        trait_id: "".to_string(),
                    }),
                    ..Default::default()
                }
                .pack_to_any()?,
            ),
            ..Default::default()
        },
    });
    let et2 = IndexOperation::PutTrait(PutTraitMutation {
        block_offset: None,
        operation_id: 2,
        entity_id: "et2".to_string(),
        trt: Trait {
            id: "trt2".to_string(),
            message: Some(
                TestMessage {
                    string1: "Hello World".to_string(),
                    ref1: Some(Reference {
                        entity_id: "et1".to_string(),
                        trait_id: "trt1".to_string(),
                    }),
                    ..Default::default()
                }
                .pack_to_any()?,
            ),
            ..Default::default()
        },
    });

    let long_id = "a".repeat(50);
    let et3 = IndexOperation::PutTrait(PutTraitMutation {
        block_offset: None,
        operation_id: 3,
        entity_id: long_id.clone(),
        trt: Trait {
            id: "trt3".to_string(),
            message: Some(
                TestMessage {
                    string1: "Hello World".to_string(),
                    ref1: Some(Reference {
                        entity_id: long_id.clone(),
                        ..Default::default()
                    }),
                    ..Default::default()
                }
                .pack_to_any()?,
            ),
            ..Default::default()
        },
    });
    index.apply_operations(vec![et1, et2, et3].into_iter())?;

    let search = |entity: &str, trt: &str| {
        let query = Q::references((entity, trt)).build();
        index.search(query).unwrap()
    };

    {
        // all fields search
        let res = search("et1", "");
        assert_eq!(res.mutations.len(), 1);
        find_put_trait(&res, "trt1");

        let res = search("et1", "trt1");
        assert_eq!(res.mutations.len(), 1);
        find_put_trait(&res, "trt1");

        let res = search("et1", "trt2");
        assert_eq!(res.mutations.len(), 0);

        let res = search("trt1", "");
        assert_eq!(res.mutations.len(), 0);

        let res = search("et0", "trt1");
        assert_eq!(res.mutations.len(), 0);

        let res = search("et2", "");
        assert_eq!(res.mutations.len(), 1);
        find_put_trait(&res, "trt2");
    }

    {
        // specific trait field search
        let query = Q::with_trait_name_query(
            "exocore.test.TestMessage",
            TQ::field_references("ref1", "et1").build(),
        );
        let res = index.search(query.build())?;
        assert_eq!(res.mutations.len(), 1);
        find_put_trait(&res, "trt1");

        let query = Q::with_trait_name_query(
            "exocore.test.TestMessage",
            TQ::field_references("ref1", ("et1", "trt1")).build(),
        );
        let res = index.search(query.build())?;
        assert_eq!(res.mutations.len(), 1);
        find_put_trait(&res, "trt1");

        let query = Q::with_trait_name_query(
            "exocore.test.TestMessage",
            TQ::field_references("ref1", ("et1", "trt2")).build(),
        );
        let res = index.search(query.build())?;
        assert_eq!(res.mutations.len(), 0);
    }

    {
        // search for long ids (tantivy default text is length 40 max)
        let query = Q::with_trait_name_query(
            "exocore.test.TestMessage",
            TQ::field_references("ref1", long_id).build(),
        );
        let res = index.search(query.build())?;
        assert_eq!(res.mutations.len(), 1);
        find_put_trait(&res, "trt3");
    }

    Ok(())
}

#[test]
fn search_by_operations() -> anyhow::Result<()> {
    let registry = Arc::new(Registry::new_with_exocore_types());
    let config = test_config();
    let mut index = MutationIndex::create_in_memory(config, registry)?;

    let et1 = IndexOperation::PutTrait(PutTraitMutation {
        block_offset: None,
        operation_id: 1,
        entity_id: "et1".to_string(),
        trt: Trait {
            id: "trt1".to_string(),
            message: Some(
                TestMessage {
                    string1: "Foo Bar".to_string(),
                    ref1: Some(Reference {
                        entity_id: "et2".to_string(),
                        trait_id: "".to_string(),
                    }),
                    ..Default::default()
                }
                .pack_to_any()?,
            ),
            ..Default::default()
        },
    });
    let et2 = IndexOperation::PutTrait(PutTraitMutation {
        block_offset: None,
        operation_id: 2,
        entity_id: "et2".to_string(),
        trt: Trait {
            id: "trt2".to_string(),
            message: Some(
                TestMessage {
                    string1: "Hello World".to_string(),
                    ref1: Some(Reference {
                        entity_id: "et1".to_string(),
                        trait_id: "trt1".to_string(),
                    }),
                    ..Default::default()
                }
                .pack_to_any()?,
            ),
            ..Default::default()
        },
    });
    index.apply_operations(vec![et1, et2].into_iter())?;

    let search = |operations: Vec<OperationId>| {
        let query = Q::with_operations(operations).build();
        index.search(query).unwrap()
    };

    {
        let res = search(vec![1]);
        assert_eq!(res.mutations.len(), 1);
        find_put_trait(&res, "trt1");

        let res = search(vec![2]);
        assert_eq!(res.mutations.len(), 1);
        find_put_trait(&res, "trt2");

        let res = search(vec![1, 2]);
        assert_eq!(res.mutations.len(), 2);
        find_put_trait(&res, "trt1");
        find_put_trait(&res, "trt2");

        let res = search(vec![3]);
        assert_eq!(res.mutations.len(), 0);

        let res = search(vec![]);
        assert_eq!(res.mutations.len(), 0);

        let res = search(vec![1, 3]);
        assert_eq!(res.mutations.len(), 1);
        find_put_trait(&res, "trt1");
    }

    Ok(())
}

#[test]
fn search_by_ids() -> anyhow::Result<()> {
    let registry = Arc::new(Registry::new_with_exocore_types());
    let config = test_config();
    let mut index = MutationIndex::create_in_memory(config, registry)?;

    let et1 = IndexOperation::PutTrait(PutTraitMutation {
        block_offset: None,
        operation_id: 1,
        entity_id: "et1".to_string(),
        trt: Trait {
            id: "trt1".to_string(),
            message: Some(
                TestMessage {
                    string1: "Foo Bar".to_string(),
                    ref1: Some(Reference {
                        entity_id: "et2".to_string(),
                        trait_id: "".to_string(),
                    }),
                    ..Default::default()
                }
                .pack_to_any()?,
            ),
            ..Default::default()
        },
    });
    let et2 = IndexOperation::PutTrait(PutTraitMutation {
        block_offset: None,
        operation_id: 2,
        entity_id: "et2".to_string(),
        trt: Trait {
            id: "trt2".to_string(),
            message: Some(
                TestMessage {
                    string1: "Hello World".to_string(),
                    ref1: Some(Reference {
                        entity_id: "et1".to_string(),
                        trait_id: "trt1".to_string(),
                    }),
                    ..Default::default()
                }
                .pack_to_any()?,
            ),
            ..Default::default()
        },
    });
    index.apply_operations(vec![et1, et2].into_iter())?;

    let search = |ids: Vec<String>| {
        let query = Q::with_ids(ids.iter()).build();
        index.search(query).unwrap()
    };

    {
        let res = search(vec!["et1".to_string()]);
        assert_eq!(res.mutations.len(), 1);
        find_put_trait(&res, "trt1");

        let res = search(vec!["et2".to_string()]);
        assert_eq!(res.mutations.len(), 1);
        find_put_trait(&res, "trt2");

        let res = search(vec!["et1".to_string(), "et2".to_string()]);
        assert_eq!(res.mutations.len(), 2);
        find_put_trait(&res, "trt1");
        find_put_trait(&res, "trt2");

        let res = search(vec!["dontexists".to_string()]);
        assert_eq!(res.mutations.len(), 0);

        let res = search(vec![]);
        assert_eq!(res.mutations.len(), 0);
    }

    Ok(())
}

#[test]
fn search_by_trait_field() -> anyhow::Result<()> {
    let registry = Arc::new(Registry::new_with_exocore_types());
    let config = test_config();
    let mut index = MutationIndex::create_in_memory(config, registry)?;

    let et1 = IndexOperation::PutTrait(PutTraitMutation {
        block_offset: None,
        operation_id: 1,
        entity_id: "et1".to_string(),
        trt: Trait {
            id: "trt1".to_string(),
            message: Some(
                TestMessage {
                    string3: "foo".to_string(),
                    ..Default::default()
                }
                .pack_to_any()?,
            ),
            ..Default::default()
        },
    });
    let et2 = IndexOperation::PutTrait(PutTraitMutation {
        block_offset: None,
        operation_id: 2,
        entity_id: "et2".to_string(),
        trt: Trait {
            id: "trt2".to_string(),
            message: Some(
                TestMessage {
                    string3: "bar".to_string(),
                    ..Default::default()
                }
                .pack_to_any()?,
            ),
            ..Default::default()
        },
    });
    index.apply_operations(vec![et1, et2].into_iter())?;

    let query = Q::with_trait_name_query(
        "exocore.test.TestMessage",
        TQ::field_equals("string3", "foo").build(),
    );
    let res = index.search(query.build())?;
    assert_eq!(res.mutations.len(), 1);
    find_put_trait(&res, "trt1");

    let query = Q::with_trait_name_query(
        "exocore.test.TestMessage",
        TQ::field_equals("string3", "bar").build(),
    );
    let res = index.search(query.build())?;
    assert_eq!(res.mutations.len(), 1);
    find_put_trait(&res, "trt2");

    Ok(())
}

#[test]
fn search_all() -> anyhow::Result<()> {
    let registry = Arc::new(Registry::new_with_exocore_types());
    let config = test_config();
    let mut index = MutationIndex::create_in_memory(config, registry)?;

    let mut mutations = Vec::new();
    for i in 0..10 {
        mutations.push(IndexOperation::PutTrait(PutTraitMutation {
            block_offset: None,
            operation_id: i,
            entity_id: format!("et{}", i),
            trt: Trait {
                id: "trt1".to_string(),
                message: Some(TestMessage::default().pack_to_any()?),
                ..Default::default()
            },
        }));
    }
    index.apply_operations(mutations.into_iter())?;

    let res = index.search(Q::all().build()).unwrap();
    assert_eq!(res.total, 10);
    assert_eq!(res.mutations.len(), 7);
    assert_eq!(res.mutations[0].operation_id, 9);
    assert_eq!(res.mutations[6].operation_id, 3);

    let next_page = res.next_page.unwrap();
    let res = index
        .search(Q::all().with_paging(next_page).build())
        .unwrap();
    assert_eq!(res.mutations.len(), 3);
    assert_eq!(res.mutations[0].operation_id, 2);
    assert_eq!(res.mutations[2].operation_id, 0);

    let res = index
        .search(Q::all().order_by_operations(true).build())
        .unwrap();
    assert_eq!(res.mutations[0].operation_id, 0);
    assert_eq!(res.mutations[1].operation_id, 1);

    Ok(())
}

#[test]
fn highest_indexed_block() -> anyhow::Result<()> {
    let registry = Arc::new(Registry::new_with_exocore_types());
    let config = test_config();
    let mut index = MutationIndex::create_in_memory(config, registry)?;

    assert_eq!(index.highest_indexed_block()?, None);

    index.apply_operation(IndexOperation::PutTrait(PutTraitMutation {
        block_offset: Some(1234),
        operation_id: 1,
        entity_id: "et1".to_string(),
        trt: Trait {
            id: "trt1".to_string(),
            message: Some(
                TestMessage {
                    string1: "Some Subject".to_string(),
                    ..Default::default()
                }
                .pack_to_any()?,
            ),
            ..Default::default()
        },
    }))?;
    assert_eq!(index.highest_indexed_block()?, Some(1234));

    index.apply_operation(IndexOperation::PutTrait(PutTraitMutation {
        block_offset: Some(120),
        operation_id: 2,
        entity_id: "et1".to_string(),
        trt: Trait {
            id: "trt2".to_string(),
            message: Some(
                TestMessage {
                    string1: "Some Subject".to_string(),
                    ..Default::default()
                }
                .pack_to_any()?,
            ),
            ..Default::default()
        },
    }))?;
    assert_eq!(index.highest_indexed_block()?, Some(1234));

    index.apply_operation(IndexOperation::PutTrait(PutTraitMutation {
        block_offset: Some(9999),
        operation_id: 3,
        entity_id: "et1".to_string(),
        trt: Trait {
            id: "trt1".to_string(),
            message: Some(
                TestMessage {
                    string1: "Some Subject".to_string(),
                    ..Default::default()
                }
                .pack_to_any()?,
            ),
            ..Default::default()
        },
    }))?;
    assert_eq!(index.highest_indexed_block()?, Some(9999));

    Ok(())
}

#[test]
fn put_unregistered_trait() -> anyhow::Result<()> {
    let registry = Arc::new(Registry::new_with_exocore_types());
    let config = test_config();
    let mut index = MutationIndex::create_in_memory(config, registry)?;

    assert_eq!(index.highest_indexed_block()?, None);

    index.apply_operation(IndexOperation::PutTrait(PutTraitMutation {
        block_offset: Some(1234),
        operation_id: 1,
        entity_id: "et1".to_string(),
        trt: Trait {
            id: "trt1".to_string(),
            message: Some(Any {
                type_url: "type.googleapis.com/not.registered.Message".to_string(),
                value: Vec::new(),
            }),
            ..Default::default()
        },
    }))?;

    let query = Q::with_trait_name("not.registered.Message").build();
    let res = index.search(query)?;
    assert_eq!(res.mutations.len(), 1);

    Ok(())
}

#[test]
fn delete_operation_id_mutation() -> anyhow::Result<()> {
    let registry = Arc::new(Registry::new_with_exocore_types());
    let config = test_config();
    let mut index = MutationIndex::create_in_memory(config, registry)?;

    let trait1 = IndexOperation::PutTrait(PutTraitMutation {
        block_offset: None,
        operation_id: 1234,
        entity_id: "entity_id1".to_string(),
        trt: Trait {
            id: "foo1".to_string(),
            message: Some(
                TestMessage {
                    string1: "Foo Bar".to_string(),
                    ..Default::default()
                }
                .pack_to_any()?,
            ),
            ..Default::default()
        },
    });
    index.apply_operation(trait1)?;

    let query = Q::matches("foo").build();
    let res = index.search(query)?;
    assert_eq!(res.mutations.len(), 1);

    index.apply_operation(IndexOperation::DeleteOperation(1234))?;

    let query = Q::matches("foo").build();
    let res = index.search(query)?;
    assert_eq!(res.mutations.len(), 0);

    Ok(())
}

#[test]
fn put_trait_tombstone() -> anyhow::Result<()> {
    let registry = Arc::new(Registry::new_with_exocore_types());
    let config = test_config();
    let mut index = MutationIndex::create_in_memory(config, registry)?;

    let contact_mutation = IndexOperation::PutTraitTombstone(PutTraitTombstoneMutation {
        block_offset: None,
        operation_id: 1234,
        entity_id: "entity_id1".to_string(),
        trait_id: "foo1".to_string(),
    });
    index.apply_operation(contact_mutation)?;

    let trait1 = IndexOperation::PutTrait(PutTraitMutation {
        block_offset: None,
        operation_id: 2345,
        entity_id: "entity_id2".to_string(),
        trt: Trait {
            id: "foo2".to_string(),
            message: Some(
                TestMessage {
                    string1: "Foo Bar".to_string(),
                    ..Default::default()
                }
                .pack_to_any()?,
            ),
            ..Default::default()
        },
    });
    index.apply_operation(trait1)?;

    let query = Q::with_id("entity_id1").build();
    let res = index.search(query)?;
    assert_is_trait_tombstone(&res.mutations.first().unwrap().mutation_type, "foo1");

    let query = Q::with_id("entity_id2").build();
    let res = index.search(query)?;
    assert_is_put_trait(&res.mutations.first().unwrap().mutation_type, "foo2");

    Ok(())
}

#[test]
fn put_entity_tombstone() -> anyhow::Result<()> {
    let registry = Arc::new(Registry::new_with_exocore_types());
    let config = test_config();
    let mut index = MutationIndex::create_in_memory(config, registry)?;

    let trait1 = IndexOperation::PutEntityTombstone(PutEntityTombstoneMutation {
        block_offset: None,
        operation_id: 1234,
        entity_id: "entity_id1".to_string(),
    });
    index.apply_operation(trait1)?;

    let query = Q::with_id("entity_id1").build();
    let res = index.search(query)?;
    assert_is_entity_tombstone(&res.mutations.first().unwrap().mutation_type);

    Ok(())
}

#[test]
fn trait_dates() -> anyhow::Result<()> {
    let registry = Arc::new(Registry::new_with_exocore_types());
    let config = test_config();
    let mut index = MutationIndex::create_in_memory(config, registry)?;

    let creation_date = "2019-08-01T12:00:00Z".parse::<DateTime<Utc>>()?;
    let modification_date = "2019-12-03T12:00:00Z".parse::<DateTime<Utc>>()?;

    let trait1 = IndexOperation::PutTrait(PutTraitMutation {
        block_offset: Some(1),
        operation_id: 10,
        entity_id: "entity_id1".to_string(),
        trt: Trait {
            id: "foo1".to_string(),
            message: Some(
                TestMessage {
                    string1: "Foo Foo Foo Foo Bar".to_string(),
                    ..Default::default()
                }
                .pack_to_any()?,
            ),
            creation_date: Some(creation_date.to_proto_timestamp()),
            modification_date: Some(modification_date.to_proto_timestamp()),
            deletion_date: None,
            last_operation_id: 10,
            details: TraitDetails::Full.into(),
        },
    });
    index.apply_operation(trait1)?;

    let query = Q::with_id("entity_id1").build();
    let res = index.search(query)?;
    let trait_meta = find_put_trait(&res, "foo1").unwrap();
    let trait_put = assert_is_put_trait(&trait_meta.mutation_type, "foo1");
    assert_eq!(creation_date, trait_put.creation_date.unwrap());
    assert_eq!(modification_date, trait_put.modification_date.unwrap());

    Ok(())
}

fn test_config() -> MutationIndexConfig {
    MutationIndexConfig {
        iterator_page_size: 7,
        ..MutationIndexConfig::default()
    }
}

fn find_put_trait<'r>(
    results: &'r MutationResults,
    trait_id: &str,
) -> Option<&'r MutationMetadata> {
    results.mutations.iter().find(|t| matches!(&t.mutation_type, MutationType::TraitPut(put_trait) if put_trait.trait_id == trait_id))
}

fn assert_is_put_trait<'r>(
    document_type: &'r MutationType,
    trait_id: &str,
) -> &'r PutTraitMetadata {
    match document_type {
        MutationType::TraitPut(put_trait) if put_trait.trait_id == trait_id => put_trait,
        other => panic!("Expected TraitPut type, but got {:?}", other),
    }
}

fn assert_is_trait_tombstone(document_type: &MutationType, trait_id: &str) {
    match document_type {
        MutationType::TraitTombstone(trt_id) if trt_id == trait_id => {}
        other => panic!("Expected TraitTombstone type, but got {:?}", other),
    }
}

fn assert_is_entity_tombstone(document_type: &MutationType) {
    match document_type {
        MutationType::EntityTombstone => {}
        other => panic!("Expected EntityTombstone type, but got {:?}", other),
    }
}

fn extract_traits_id(results: &MutationResults) -> Vec<String> {
    results
        .mutations
        .iter()
        .map(|res| match &res.mutation_type {
            MutationType::TraitPut(put_trait) => put_trait.trait_id.clone(),
            other => panic!("Expected trait put, got something else: {:?}", other),
        })
        .collect()
}
