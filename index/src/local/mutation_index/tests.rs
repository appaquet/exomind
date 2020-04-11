use chrono::{DateTime, Utc};
use itertools::Itertools;

use exocore_core::protos::generated::exocore_index::Trait;
use exocore_core::protos::generated::exocore_test::{TestMessage, TestMessage2};
use exocore_core::protos::prost::{Any, ProstAnyPackMessageExt, ProstDateTimeExt};

use crate::query::QueryBuilder;

use super::*;

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

    let results = index.search_matches("foo", None)?;
    assert_eq!(results.results.len(), 2);
    assert_eq!(results.results[0].entity_id, "entity_id1"); // foo is repeated in entity 1

    let results = index.search_matches("bar", None)?;
    assert_eq!(results.results.len(), 2);
    assert!(results.results[0].score > score_to_u64(0.30));
    assert!(results.results[1].score > score_to_u64(0.18));
    assert_eq!(results.results[0].entity_id, "entity_id2"); // foo is repeated in entity 2

    // with limit
    let paging = QueryPaging {
        after_score: None,
        before_score: None,
        count: 1,
    };
    let results = index.search_matches("foo", Some(paging))?;
    assert_eq!(results.results.len(), 1);
    assert_eq!(results.remaining_results, 1);
    assert_eq!(results.total_results, 2);

    // only results from given score
    let paging = QueryPaging {
        after_score: Some(score_to_u64(0.30)),
        before_score: None,
        count: 10,
    };
    let results = index.search_matches("bar", Some(paging))?;
    assert_eq!(results.results.len(), 1);
    assert_eq!(results.remaining_results, 0);
    assert_eq!(results.total_results, 2);
    assert_eq!(results.results[0].entity_id, "entity_id2");

    // only results before given score
    let paging = QueryPaging {
        after_score: None,
        before_score: Some(score_to_u64(0.30)),
        count: 10,
    };
    let results = index.search_matches("bar", Some(paging))?;
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
        IndexMutation::PutTrait(PutTraitMutation {
            block_offset: Some(i),
            operation_id: i,
            entity_id: format!("entity_id{}", i),
            trt: Trait {
                id: format!("entity_id{}", i),
                message: Some(
                    TestMessage {
                        string1: "Foo Bar".to_string(),
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

    let paging = QueryPaging {
        after_score: None,
        before_score: None,
        count: 10,
    };
    let results1 = index.search_matches("foo", Some(paging))?;
    assert_eq!(results1.total_results, 30);
    assert_eq!(results1.results.len(), 10);
    assert_eq!(results1.remaining_results, 20);
    find_put_trait(&results1, "id29");
    find_put_trait(&results1, "id20");

    let results2 = index.search_matches("foo", Some(results1.next_page.clone().unwrap()))?;
    assert_eq!(results2.total_results, 30);
    assert_eq!(results2.results.len(), 10);
    assert_eq!(results2.remaining_results, 10);
    find_put_trait(&results1, "id19");
    find_put_trait(&results1, "id10");

    let results3 = index.search_matches("foo", Some(results2.next_page.unwrap()))?;
    assert_eq!(results3.total_results, 30);
    assert_eq!(results3.results.len(), 10);
    assert_eq!(results3.remaining_results, 0);
    find_put_trait(&results1, "id9");
    find_put_trait(&results1, "id0");

    // search all should return an iterator over all results
    let query = QueryBuilder::match_text("foo").build();
    let iter = index.search_all(&query)?;
    assert_eq!(iter.total_results, 30);
    let results = iter.collect_vec();
    assert_eq!(results.len(), 30);

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

    let results = index.search_with_trait("exocore.test.TestMessage", None)?;
    assert_eq!(results.results.len(), 1);
    assert!(find_put_trait(&results, "trt1").is_some());

    // ordering of multiple traits is operation id
    let results = index.search_with_trait("exocore.test.TestMessage2", None)?;
    assert_eq!(
        extract_traits_id(results),
        vec!["trait4", "trait3", "trait2"]
    );

    // with limit
    let paging = QueryPaging {
        after_score: None,
        before_score: None,
        count: 1,
    };
    let results = index.search_with_trait("exocore.test.TestMessage2", Some(paging))?;
    assert_eq!(results.results.len(), 1);

    // only results after given modification date
    let paging = QueryPaging {
        after_score: Some(2),
        before_score: None,
        count: 10,
    };
    let results = index.search_with_trait("exocore.test.TestMessage2", Some(paging))?;
    assert_eq!(extract_traits_id(results), vec!["trait4", "trait3"]);

    // only results before given modification date
    let paging = QueryPaging {
        after_score: None,
        before_score: Some(3),
        count: 10,
    };
    let results = index.search_with_trait("exocore.test.TestMessage2", Some(paging))?;
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

    let paging = QueryPaging {
        after_score: None,
        before_score: None,
        count: 10,
    };

    let results1 = index.search_with_trait("exocore.test.TestMessage", Some(paging))?;
    assert_eq!(results1.total_results, 30);
    assert_eq!(results1.remaining_results, 20);
    assert_eq!(results1.results.len(), 10);
    find_put_trait(&results1, "id29");
    find_put_trait(&results1, "id20");

    let results2 = index.search_with_trait(
        "exocore.test.TestMessage",
        Some(results1.next_page.clone().unwrap()),
    )?;
    assert_eq!(results2.total_results, 30);
    assert_eq!(results2.remaining_results, 10);
    assert_eq!(results2.results.len(), 10);
    find_put_trait(&results1, "id19");
    find_put_trait(&results1, "id10");

    let results3 = index.search_with_trait(
        "exocore.test.TestMessage",
        Some(results2.next_page.unwrap()),
    )?;
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

    let results = index.search_with_trait("not.registered.Message", None)?;
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

    assert_eq!(index.search_matches("foo", None)?.results.len(), 1);

    index.apply_mutation(IndexMutation::DeleteOperation(1234))?;

    assert_eq!(index.search_matches("foo", None)?.results.len(), 0);

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
