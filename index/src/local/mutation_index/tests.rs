use chrono::{DateTime, Utc};
use itertools::Itertools;

use exocore_core::protos::generated::exocore_index::{Reference, Trait};
use exocore_core::protos::generated::exocore_test::{TestMessage, TestMessage2};
use exocore_core::protos::prost::{Any, ProstAnyPackMessageExt, ProstDateTimeExt};

use crate::query::QueryBuilder;

use super::*;
use exocore_chain::operation::OperationId;

// TODO: Sort by operation, ascending, descending
// TODO: Sort by match, acending, descending
// TODO: Trait inner query support
//       -> Sort by field, ascending, descending
// TODO: SortToken check if it's too cloned
// TODO: Top docs collector could use references ?

#[test]
fn search_by_entity_id() -> Result<(), failure::Error> {
    let registry = Arc::new(Registry::new_with_exocore_types());
    let config = test_config();
    let mut index = MutationIndex::create_in_memory(config, registry)?;

    let trait1 = IndexMutation::PutTrait(PutTraitMutation {
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

    let trait2 = IndexMutation::PutTrait(PutTraitMutation {
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

    let trait3 = IndexMutation::PutTrait(PutTraitMutation {
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
    index.apply_mutations(vec![trait1, trait2, trait3].into_iter())?;

    let results = index.search_entity_id("entity_id1")?;
    assert_eq!(results.results.len(), 1);
    assert_eq!(results.results[0].block_offset, Some(1));
    assert_eq!(results.results[0].operation_id, 10);
    assert_eq!(results.results[0].entity_id, "entity_id1");
    assert_is_put_trait(&results.results[0].mutation_type, "foo1");

    let results = index.search_entity_id("entity_id2")?;
    assert_eq!(results.results.len(), 2);
    find_put_trait(&results, "foo2");
    find_put_trait(&results, "foo3");

    // search all should return an iterator all results
    let query = QueryBuilder::with_entity_id("entity_id2").build();
    let iter = index.search_all(&query)?;
    assert_eq!(iter.total_results, 2);
    let results = iter.collect_vec();
    assert_eq!(results.len(), 2);

    Ok(())
}

#[test]
fn search_query_matches() -> Result<(), failure::Error> {
    let registry = Arc::new(Registry::new_with_exocore_types());
    let config = test_config();
    let mut index = MutationIndex::create_in_memory(config, registry)?;

    let trait1 = IndexMutation::PutTrait(PutTraitMutation {
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

    let trait2 = IndexMutation::PutTrait(PutTraitMutation {
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
    index.apply_mutations(vec![trait1, trait2].into_iter())?;

    let predicate = |text: &str| MatchPredicate {
        query: text.to_string(),
    };

    let results = index.search_matches(&predicate("foo"), None, None)?;
    assert_eq!(results.results.len(), 2);
    assert_eq!(results.results[0].entity_id, "entity_id1"); // foo is repeated in entity 1

    let results = index.search_matches(&predicate("bar"), None, None)?;
    assert_eq!(results.results.len(), 2);

    assert!(results.results[0].sort_token > SortToken::from_f32(0.30));
    assert!(results.results[1].sort_token > SortToken::from_f32(0.18));
    assert_eq!(results.results[0].entity_id, "entity_id2"); // foo is repeated in entity 2

    // with limit
    let paging = Paging {
        after_token: String::new(),
        before_token: String::new(),
        count: 1,
    };
    let results = index.search_matches(&predicate("foo"), Some(paging), None)?;
    assert_eq!(results.results.len(), 1);
    assert_eq!(results.remaining_results, 1);
    assert_eq!(results.total_results, 2);

    // only results from given score
    let paging = Paging {
        after_token: SortToken::from_f32(0.30).into(),
        before_token: String::new(),
        count: 10,
    };
    let results = index.search_matches(&predicate("bar"), Some(paging), None)?;
    assert_eq!(results.results.len(), 1);
    assert_eq!(results.remaining_results, 0);
    assert_eq!(results.total_results, 2);
    assert_eq!(results.results[0].entity_id, "entity_id2");

    // only results before given score
    let paging = Paging {
        after_token: String::new(),
        before_token: SortToken::from_f32(0.30).into(),
        count: 10,
    };
    let results = index.search_matches(&predicate("bar"), Some(paging), None)?;
    assert_eq!(results.results.len(), 1);
    assert_eq!(results.remaining_results, 0);
    assert_eq!(results.total_results, 2);
    assert_eq!(results.results[0].entity_id, "entity_id1");

    Ok(())
}

#[test]
fn search_query_matches_paging() -> Result<(), failure::Error> {
    let registry = Arc::new(Registry::new_with_exocore_types());
    let config = test_config();
    let mut index = MutationIndex::create_in_memory(config, registry)?;

    let traits = (0..30).map(|i| {
        let text = "foo ".repeat((i + 1) as usize);

        IndexMutation::PutTrait(PutTraitMutation {
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
    index.apply_mutations(traits)?;

    let paging = Paging {
        after_token: String::new(),
        before_token: String::new(),
        count: 10,
    };

    let query = QueryBuilder::match_text("foo").with_paging(paging).build();
    let results1 = index.search(&query)?;
    let results1_next_page = results1.next_page.clone().unwrap();
    assert_eq!(results1.total_results, 30);
    assert_eq!(results1.results.len(), 10);
    assert_eq!(results1.remaining_results, 20);
    let ids = extract_traits_id(results1);
    assert_eq!(ids[0], "id29");
    assert_eq!(ids[9], "id20");

    let query = QueryBuilder::match_text("foo")
        .with_paging(results1_next_page)
        .build();
    let results2 = index.search(&query)?;
    let results2_next_page = results2.next_page.clone().unwrap();
    assert_eq!(results2.total_results, 30);
    assert_eq!(results2.results.len(), 10);
    assert_eq!(results2.remaining_results, 10);
    let ids = extract_traits_id(results2);
    assert_eq!(ids[0], "id19");
    assert_eq!(ids[9], "id10");

    let query = QueryBuilder::match_text("foo")
        .with_paging(results2_next_page)
        .build();
    let results3 = index.search(&query)?;
    assert_eq!(results3.total_results, 30);
    assert_eq!(results3.results.len(), 10);
    assert_eq!(results3.remaining_results, 0);
    let ids = extract_traits_id(results3);
    assert_eq!(ids[0], "id9");
    assert_eq!(ids[9], "id0");

    // search all should return an iterator over all results
    let query = QueryBuilder::match_text("foo").build();
    let iter = index.search_all(&query)?;
    assert_eq!(iter.total_results, 30);
    let ops: Vec<OperationId> = iter.map(|r| r.operation_id).collect();
    assert_eq!(ops.len(), 30);
    assert_eq!(ops[0], 29);
    assert_eq!(ops[29], 0);

    // reversed order
    let query = QueryBuilder::match_text("foo").order(true).build();
    let iter = index.search_all(&query)?;
    assert_eq!(iter.total_results, 30);
    let ops: Vec<OperationId> = iter.map(|r| r.operation_id).collect();
    assert_eq!(ops.len(), 30);
    assert_eq!(ops[0], 0);
    assert_eq!(ops[29], 29);

    Ok(())
}

#[test]
fn search_query_by_trait_type() -> Result<(), failure::Error> {
    let registry = Arc::new(Registry::new_with_exocore_types());
    let config = test_config();
    let mut index = MutationIndex::create_in_memory(config, registry)?;

    let trait1 = IndexMutation::PutTrait(PutTraitMutation {
        block_offset: None,
        operation_id: 1,
        entity_id: "entity_id1".to_string(),
        trt: Trait {
            id: "trt1".to_string(),
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

    let trait2 = IndexMutation::PutTrait(PutTraitMutation {
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

    let trait3 = IndexMutation::PutTrait(PutTraitMutation {
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

    let trait4 = IndexMutation::PutTrait(PutTraitMutation {
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

    index.apply_mutations(vec![trait4, trait3, trait2, trait1].into_iter())?;

    let pred1 = TraitPredicate {
        trait_name: "exocore.test.TestMessage".to_string(),
        query: None,
    };
    let pred2 = TraitPredicate {
        trait_name: "exocore.test.TestMessage2".to_string(),
        query: None,
    };

    let results = index.search_with_trait(&pred1, None, None)?;
    assert_eq!(results.results.len(), 1);
    assert!(find_put_trait(&results, "trt1").is_some());

    // ordering of multiple traits is operation id
    let results = index.search_with_trait(&pred2, None, None)?;
    assert_eq!(
        extract_traits_id(results),
        vec!["trait4", "trait3", "trait2"]
    );

    // with limit
    let paging = Paging {
        after_token: String::new(),
        before_token: String::new(),
        count: 1,
    };
    let results = index.search_with_trait(&pred2, Some(paging), None)?;
    assert_eq!(results.results.len(), 1);

    // only results after given modification date
    let paging = Paging {
        after_token: SortToken::from_u64(2).into(),
        before_token: String::new(),
        count: 10,
    };
    let results = index.search_with_trait(&pred2, Some(paging), None)?;
    assert_eq!(extract_traits_id(results), vec!["trait4", "trait3"]);

    // only results before given modification date
    let paging = Paging {
        after_token: String::new(),
        before_token: SortToken::from_u64(3).into(),
        count: 10,
    };
    let results = index.search_with_trait(&pred2, Some(paging), None)?;
    assert_eq!(extract_traits_id(results), vec!["trait2"]);

    Ok(())
}

#[test]
fn search_query_by_trait_type_paging() -> Result<(), failure::Error> {
    let registry = Arc::new(Registry::new_with_exocore_types());
    let config = test_config();
    let mut index = MutationIndex::create_in_memory(config, registry)?;

    let traits = (0..30).map(|i| {
        IndexMutation::PutTrait(PutTraitMutation {
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
    index.apply_mutations(traits)?;

    let paging = Paging {
        after_token: String::new(),
        before_token: String::new(),
        count: 10,
    };

    let pred1 = TraitPredicate {
        trait_name: "exocore.test.TestMessage".to_string(),
        query: None,
    };

    let results1 = index.search_with_trait(&pred1, Some(paging), None)?;
    assert_eq!(results1.total_results, 30);
    assert_eq!(results1.remaining_results, 20);
    assert_eq!(results1.results.len(), 10);
    find_put_trait(&results1, "id29");
    find_put_trait(&results1, "id20");

    let results2 =
        index.search_with_trait(&pred1, Some(results1.next_page.clone().unwrap()), None)?;
    assert_eq!(results2.total_results, 30);
    assert_eq!(results2.remaining_results, 10);
    assert_eq!(results2.results.len(), 10);
    find_put_trait(&results1, "id19");
    find_put_trait(&results1, "id10");

    let results3 = index.search_with_trait(&pred1, Some(results2.next_page.unwrap()), None)?;
    assert_eq!(results3.total_results, 30);
    assert_eq!(results3.remaining_results, 0);
    assert_eq!(results3.results.len(), 10);
    find_put_trait(&results1, "id9");
    find_put_trait(&results1, "id0");

    // search all should return an iterator over all results
    let query = QueryBuilder::with_trait("exocore.test.TestMessage").build();
    let iter = index.search_all(&query)?;
    assert_eq!(iter.total_results, 30);
    let results = iter.collect_vec();
    assert_eq!(results.len(), 30);

    Ok(())
}

#[test]
fn search_by_reference() -> Result<(), failure::Error> {
    let registry = Arc::new(Registry::new_with_exocore_types());
    let config = test_config();
    let mut index = MutationIndex::create_in_memory(config, registry)?;

    let et1 = IndexMutation::PutTrait(PutTraitMutation {
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
    let et2 = IndexMutation::PutTrait(PutTraitMutation {
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
    index.apply_mutations(vec![et1, et2].into_iter())?;

    let search = |entity: &str, trt: &str| {
        index
            .search_reference(
                &ReferencePredicate {
                    entity_id: entity.to_string(),
                    trait_id: trt.to_string(),
                },
                None,
                None,
            )
            .unwrap()
    };

    let results = search("et1", "");
    assert_eq!(results.results.len(), 1);
    find_put_trait(&results, "trt1");

    let results = search("et1", "trt1");
    assert_eq!(results.results.len(), 1);
    find_put_trait(&results, "trt1");

    let results = search("et1", "trt2");
    assert_eq!(results.results.len(), 0);

    let results = search("trt1", "");
    assert_eq!(results.results.len(), 0);

    let results = search("et0", "trt1");
    assert_eq!(results.results.len(), 0);

    let results = search("et2", "");
    assert_eq!(results.results.len(), 1);
    find_put_trait(&results, "trt2");

    Ok(())
}

#[test]
fn highest_indexed_block() -> Result<(), failure::Error> {
    let registry = Arc::new(Registry::new_with_exocore_types());
    let config = test_config();
    let mut index = MutationIndex::create_in_memory(config, registry)?;

    assert_eq!(index.highest_indexed_block()?, None);

    index.apply_mutation(IndexMutation::PutTrait(PutTraitMutation {
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

    index.apply_mutation(IndexMutation::PutTrait(PutTraitMutation {
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

    index.apply_mutation(IndexMutation::PutTrait(PutTraitMutation {
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
fn put_unregistered_trait() -> Result<(), failure::Error> {
    let registry = Arc::new(Registry::new_with_exocore_types());
    let config = test_config();
    let mut index = MutationIndex::create_in_memory(config, registry)?;

    assert_eq!(index.highest_indexed_block()?, None);

    index.apply_mutation(IndexMutation::PutTrait(PutTraitMutation {
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

    let pred = TraitPredicate {
        trait_name: "not.registered.Message".to_string(),
        query: None,
    };
    let results = index.search_with_trait(&pred, None, None)?;
    assert_eq!(results.results.len(), 1);

    Ok(())
}

#[test]
fn delete_operation_id_mutation() -> Result<(), failure::Error> {
    let registry = Arc::new(Registry::new_with_exocore_types());
    let config = test_config();
    let mut index = MutationIndex::create_in_memory(config, registry)?;

    let trait1 = IndexMutation::PutTrait(PutTraitMutation {
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
    index.apply_mutation(trait1)?;

    let predicate = MatchPredicate {
        query: "foo".to_string(),
    };
    let results = index.search_matches(&predicate, None, None)?;
    assert_eq!(results.results.len(), 1);

    index.apply_mutation(IndexMutation::DeleteOperation(1234))?;

    let results = index.search_matches(&predicate, None, None)?;
    assert_eq!(results.results.len(), 0);

    Ok(())
}

#[test]
fn put_trait_tombstone() -> Result<(), failure::Error> {
    let registry = Arc::new(Registry::new_with_exocore_types());
    let config = test_config();
    let mut index = MutationIndex::create_in_memory(config, registry)?;

    let contact_mutation = IndexMutation::PutTraitTombstone(PutTraitTombstone {
        block_offset: None,
        operation_id: 1234,
        entity_id: "entity_id1".to_string(),
        trait_id: "foo1".to_string(),
    });
    index.apply_mutation(contact_mutation)?;

    let trait1 = IndexMutation::PutTrait(PutTraitMutation {
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
    index.apply_mutation(trait1)?;

    let res = index.search_entity_id("entity_id1")?;
    assert_is_trait_tombstone(&res.results.first().unwrap().mutation_type, "foo1");

    let res = index.search_entity_id("entity_id2")?;
    assert_is_put_trait(&res.results.first().unwrap().mutation_type, "foo2");

    Ok(())
}

#[test]
fn put_entity_tombstone() -> Result<(), failure::Error> {
    let registry = Arc::new(Registry::new_with_exocore_types());
    let config = test_config();
    let mut index = MutationIndex::create_in_memory(config, registry)?;

    let trait1 = IndexMutation::PutEntityTombstone(PutEntityTombstone {
        block_offset: None,
        operation_id: 1234,
        entity_id: "entity_id1".to_string(),
    });
    index.apply_mutation(trait1)?;

    let res = index.search_entity_id("entity_id1")?;
    assert_is_entity_tombstone(&res.results.first().unwrap().mutation_type);

    Ok(())
}

#[test]
fn trait_dates() -> Result<(), failure::Error> {
    let registry = Arc::new(Registry::new_with_exocore_types());
    let config = test_config();
    let mut index = MutationIndex::create_in_memory(config, registry)?;

    let creation_date = "2019-08-01T12:00:00Z".parse::<DateTime<Utc>>()?;
    let modification_date = "2019-12-03T12:00:00Z".parse::<DateTime<Utc>>()?;

    let trait1 = IndexMutation::PutTrait(PutTraitMutation {
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
        },
    });
    index.apply_mutation(trait1)?;

    let res = index.search_entity_id("entity_id1")?;
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
    results.results.iter().find(|t| match &t.mutation_type {
        MutationMetadataType::TraitPut(put_trait) if put_trait.trait_id == trait_id => true,
        _ => false,
    })
}

fn assert_is_put_trait<'r>(
    document_type: &'r MutationMetadataType,
    trait_id: &str,
) -> &'r PutTraitMetadata {
    match document_type {
        MutationMetadataType::TraitPut(put_trait) if put_trait.trait_id == trait_id => put_trait,
        other => panic!("Expected TraitPut type, but got {:?}", other),
    }
}

fn assert_is_trait_tombstone(document_type: &MutationMetadataType, trait_id: &str) {
    match document_type {
        MutationMetadataType::TraitTombstone(trt_id) if trt_id == trait_id => {}
        other => panic!("Expected TraitTombstone type, but got {:?}", other),
    }
}

fn assert_is_entity_tombstone(document_type: &MutationMetadataType) {
    match document_type {
        MutationMetadataType::EntityTombstone => {}
        other => panic!("Expected EntityTombstone type, but got {:?}", other),
    }
}

fn extract_traits_id(results: MutationResults) -> Vec<String> {
    results
        .results
        .iter()
        .map(|res| match &res.mutation_type {
            MutationMetadataType::TraitPut(put_trait) => put_trait.trait_id.clone(),
            other => panic!("Expected trait put, got something else: {:?}", other),
        })
        .collect()
}
