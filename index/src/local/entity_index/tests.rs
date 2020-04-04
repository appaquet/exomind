use exocore_core::protos::generated::exocore_test::TestMessage;
use exocore_core::protos::prost::ProstTimestampExt;

use crate::mutation::MutationBuilder;
use crate::query::QueryBuilder;

use super::*;
use crate::local::mutation_index::MutationResults;

use test_index::*;

#[test]
fn index_full_pending_to_chain() -> Result<(), failure::Error> {
    let config = EntityIndexConfig {
        chain_index_min_depth: 1, // index when block is at depth 1 or more
        ..TestEntityIndex::create_test_config()
    };
    let mut test_index = TestEntityIndex::new_with_config(config)?;
    test_index.handle_engine_events()?;

    // index a few traits, they should now be available from pending index
    let first_ops_id = test_index.put_test_traits(0..=4)?;
    test_index.wait_operations_emitted(&first_ops_id);
    test_index.handle_engine_events()?;
    let res = test_index
        .index
        .search(&QueryBuilder::with_trait("exocore.test.TestMessage").build())?;
    let pending_res = count_results_source(&res, EntityResultSource::Pending);
    let chain_res = count_results_source(&res, EntityResultSource::Chain);
    assert_eq!(pending_res + chain_res, 5);

    // index a few traits, wait for first block to be committed
    let second_ops_id = test_index.put_test_traits(5..=9)?;
    test_index.wait_operations_emitted(&second_ops_id);
    test_index.wait_operations_committed(&first_ops_id);
    test_index.handle_engine_events()?;
    let res = test_index
        .index
        .search(&QueryBuilder::with_trait("exocore.test.TestMessage").build())?;
    let pending_res = count_results_source(&res, EntityResultSource::Pending);
    let chain_res = count_results_source(&res, EntityResultSource::Chain);
    assert_eq!(pending_res + chain_res, 10);

    // wait for second block to be committed, first operations should now be indexed
    // in chain
    test_index.wait_operations_committed(&second_ops_id);
    test_index.handle_engine_events()?;
    let res = test_index
        .index
        .search(&QueryBuilder::with_trait("exocore.test.TestMessage").build())?;
    let pending_res = count_results_source(&res, EntityResultSource::Pending);
    let chain_res = count_results_source(&res, EntityResultSource::Chain);
    assert!(chain_res >= 5, "was equal to {}", chain_res);
    assert_eq!(pending_res + chain_res, 10);

    Ok(())
}

#[test]
fn reopen_chain_index() -> Result<(), failure::Error> {
    let config = EntityIndexConfig {
        chain_index_min_depth: 0, // index as soon as new block appear
        chain_index_in_memory: false,
        ..TestEntityIndex::create_test_config()
    };

    // index a few traits & make sure it's in the chain index
    let mut test_index = TestEntityIndex::new_with_config(config)?;
    let ops_id = test_index.put_test_traits(0..=9)?;
    test_index.wait_operations_committed(&ops_id);
    test_index.drain_received_events();
    test_index.index.reindex_chain()?;

    // reopen index, make sure data is still in there
    let test_index = test_index.with_restarted_node()?;
    // traits should still be indexed
    let res = test_index
        .index
        .search(&QueryBuilder::with_trait("exocore.test.TestMessage").build())?;
    assert_eq!(res.entities.len(), 10);

    Ok(())
}

#[test]
fn reopen_chain_and_pending_transition() -> Result<(), failure::Error> {
    let config = EntityIndexConfig {
        chain_index_min_depth: 2,
        chain_index_in_memory: false,
        ..TestEntityIndex::create_test_config()
    };

    let mut test_index = TestEntityIndex::new_with_config(config)?;
    let query = QueryBuilder::with_trait("exocore.test.TestMessage")
        .with_count(100)
        .build();

    let mut range_from = 0;
    for i in 1..=3 {
        let range_to = range_from + 9;

        let ops_id = test_index.put_test_traits(range_from..=range_to)?;
        test_index.wait_operations_committed(&ops_id);
        test_index.handle_engine_events()?;

        let res = test_index.index.search(&query)?;
        assert_eq!(res.entities.len(), i * 10);

        // restart node, which will clear pending
        // reopening index should re-index first block in pending
        test_index = test_index.with_restarted_node()?;

        // traits should still be indexed
        let res = test_index.index.search(&query)?;
        assert_eq!(res.entities.len(), i * 10);

        range_from = range_to + 1;
    }

    Ok(())
}

#[test]
fn reindex_pending_on_discontinuity() -> Result<(), failure::Error> {
    let mut test_index = TestEntityIndex::new()?;

    // index traits without indexing them by clearing events
    test_index.put_test_traits(0..=5)?;
    test_index.drain_received_events();

    let res = test_index
        .index
        .search(&QueryBuilder::with_trait("exocore.test.TestMessage").build())?;
    assert_eq!(res.entities.len(), 0);

    // trigger discontinuity, which should force reindex
    test_index
        .index
        .handle_chain_engine_event(Event::StreamDiscontinuity)?;

    // pending is indexed
    let res = test_index
        .index
        .search(&QueryBuilder::with_trait("exocore.test.TestMessage").build())?;
    assert_eq!(res.entities.len(), 6);

    Ok(())
}

#[test]
fn chain_divergence() -> Result<(), failure::Error> {
    let config = EntityIndexConfig {
        chain_index_min_depth: 0, // index as soon as new block appear
        ..TestEntityIndex::create_test_config()
    };
    let mut test_index = TestEntityIndex::new_with_config(config)?;

    // create 3 blocks worth of traits
    let ops_id = test_index.put_test_traits(0..=2)?;
    test_index.wait_operations_committed(&ops_id);
    let ops_id = test_index.put_test_traits(3..=5)?;
    test_index.wait_operations_committed(&ops_id);
    let ops_id = test_index.put_test_traits(6..=9)?;
    test_index.wait_operations_committed(&ops_id);
    test_index.drain_received_events();

    // divergence without anything in index will trigger re-indexation
    test_index
        .index
        .handle_chain_engine_event(Event::ChainDiverged(0))?;
    let res = test_index
        .index
        .search(&QueryBuilder::with_trait("exocore.test.TestMessage").build())?;
    assert_eq!(res.entities.len(), 10);

    // divergence at an offset not indexed yet will just re-index pending
    let (chain_last_offset, _) = test_index
        .cluster
        .get_handle(0)
        .get_chain_last_block_info()?
        .unwrap();
    test_index
        .index
        .handle_chain_engine_event(Event::ChainDiverged(chain_last_offset + 1))?;
    let res = test_index
        .index
        .search(&QueryBuilder::with_trait("exocore.test.TestMessage").build())?;
    assert_eq!(res.entities.len(), 10);

    // divergence at an offset indexed in chain index will fail
    let res = test_index
        .index
        .handle_chain_engine_event(Event::ChainDiverged(0));
    assert!(res.is_err());

    Ok(())
}

#[test]
fn delete_entity_trait() -> Result<(), failure::Error> {
    let config = EntityIndexConfig {
        chain_index_min_depth: 1, // index in chain as soon as another block is after
        ..TestEntityIndex::create_test_config()
    };
    let mut test_index = TestEntityIndex::new_with_config(config)?;

    let op1 = test_index.put_test_trait("entity1", "trait1", "name1")?;
    let op2 = test_index.put_test_trait("entity1", "trait2", "name2")?;
    test_index.wait_operations_committed(&[op1, op2]);
    test_index.handle_engine_events()?;

    let entity = test_index.index.fetch_entity("entity1")?;
    assert_eq!(entity.traits.len(), 2);

    // delete trait2, this should delete via a tombstone in pending
    let op_id = test_index.delete_trait("entity1", "trait2")?;
    test_index.wait_operation_committed(op_id);
    test_index.handle_engine_events()?;
    let entity = test_index.index.fetch_entity("entity1")?;
    assert_eq!(entity.traits.len(), 1);

    let pending_res = test_index.index.pending_index.search_entity_id("entity1")?;
    assert!(pending_res.results.iter().any(|r| match &r.mutation_type {
        MutationMetadataType::TraitTombstone(_) => true,
        _ => false,
    }));

    // now bury the deletion under 1 block, which should delete for real the trait
    let op_id = test_index.put_test_trait("entity2", "trait2", "name1")?;
    test_index.wait_operation_committed(op_id);
    test_index.handle_engine_events()?;

    Ok(())
}

#[test]
fn delete_all_entity_traits() -> Result<(), failure::Error> {
    let config = TestEntityIndex::create_test_config();
    let mut test_index = TestEntityIndex::new_with_config(config)?;

    let op1 = test_index.put_test_trait("entity1", "trait1", "name1")?;
    let op2 = test_index.put_test_trait("entity1", "trait2", "name2")?;
    test_index.wait_operations_committed(&[op1, op2]);
    test_index.handle_engine_events()?;

    let query = QueryBuilder::with_entity_id("entity1").build();
    let res = test_index.index.search(&query)?;
    assert_eq!(res.entities.len(), 1);

    let op_id = test_index.delete_trait("entity1", "trait1")?;
    test_index.wait_operation_committed(op_id);
    test_index.handle_engine_events()?;

    let query = QueryBuilder::with_entity_id("entity1").build();
    let res = test_index.index.search(&query)?;
    assert_eq!(res.entities.len(), 1);

    let op_id = test_index.delete_trait("entity1", "trait2")?;
    test_index.wait_operation_committed(op_id);
    test_index.handle_engine_events()?;

    let query = QueryBuilder::with_entity_id("entity1").build();
    let res = test_index.index.search(&query)?;
    assert_eq!(res.entities.len(), 0);

    Ok(())
}

#[test]
fn delete_entity() -> Result<(), failure::Error> {
    let config = EntityIndexConfig {
        chain_index_min_depth: 1, // index in chain as soon as another block is after
        ..TestEntityIndex::create_test_config()
    };
    let mut test_index = TestEntityIndex::new_with_config(config)?;

    let op1 = test_index.put_test_trait("entity1", "trait1", "name1")?;
    let op2 = test_index.put_test_trait("entity1", "trait2", "name2")?;
    test_index.wait_operations_committed(&[op1, op2]);
    test_index.handle_engine_events()?;

    let query = QueryBuilder::with_entity_id("entity1").build();
    let res = test_index.index.search(&query)?;
    assert_eq!(res.entities.len(), 1);

    let op_id = test_index.write_mutation(MutationBuilder::delete_entity("entity1"))?;
    test_index.wait_operation_committed(op_id);
    test_index.handle_engine_events()?;
    let query = QueryBuilder::with_entity_id("entity1").build();
    let res = test_index.index.search(&query)?;
    assert_eq!(res.entities.len(), 0);

    // now bury the deletion under 1 block, which should delete for real the trait
    let op_id = test_index.put_test_trait("entity2", "trait2", "name1")?;
    test_index.wait_operation_committed(op_id);
    test_index.handle_engine_events()?;

    // should still be deleted
    let query = QueryBuilder::with_entity_id("entity1").build();
    let res = test_index.index.search(&query)?;
    assert_eq!(res.entities.len(), 0);

    Ok(())
}

#[test]
fn traits_compaction() -> Result<(), failure::Error> {
    let config = EntityIndexConfig {
        chain_index_min_depth: 1, // index in chain as soon as another block is after
        ..TestEntityIndex::create_test_config()
    };
    let mut test_index = TestEntityIndex::new_with_config(config)?;

    let op1 = test_index.put_test_trait("entity1", "trait1", "op1")?;
    let op2 = test_index.put_test_trait("entity1", "trait1", "op2")?;
    let op3 = test_index.put_test_trait("entity1", "trait1", "op3")?;
    test_index.wait_operations_committed(&[op1, op2, op3]);
    test_index.handle_engine_events()?;

    // we have 3 mutations on same trait
    let pending_res = test_index.index.pending_index.search_entity_id("entity1")?;
    let ops = extract_indexed_operations_id(pending_res);
    assert_eq!(vec![op1, op2, op3], ops);

    // mut entity has only 1 trait since all ops are on same trait
    let query = QueryBuilder::with_entity_id("entity1").build();
    let res = test_index.index.search(&query)?;
    assert_eq!(res.entities.len(), 1);
    let traits_msgs = extract_result_messages(&res.entities[0]);
    assert_eq!(traits_msgs.len(), 1);

    // last version of trait should have been ket
    assert_eq!("op3", traits_msgs[0].1.string1);

    assert_eq!(
        op1,
        traits_msgs[0]
            .0
            .creation_date
            .as_ref()
            .unwrap()
            .to_timestamp_nanos()
    );
    assert_eq!(
        op3,
        traits_msgs[0]
            .0
            .modification_date
            .as_ref()
            .unwrap()
            .to_timestamp_nanos()
    );

    let new_trait = TestEntityIndex::new_test_trait("trait1", "op4")?;
    let op_id = test_index.write_mutation(MutationBuilder::compact_traits(
        "entity1",
        new_trait,
        vec![op1, op2, op3],
    ))?;
    test_index.wait_operation_committed(op_id);
    test_index.handle_engine_events()?;

    // dates should still be the same even if we compacted the traits
    assert_eq!(
        op1,
        traits_msgs[0]
            .0
            .creation_date
            .as_ref()
            .unwrap()
            .to_timestamp_nanos()
    );
    assert_eq!(
        op3,
        traits_msgs[0]
            .0
            .modification_date
            .as_ref()
            .unwrap()
            .to_timestamp_nanos()
    );

    Ok(())
}

#[test]
fn query_paging() -> Result<(), failure::Error> {
    let config = TestEntityIndex::create_test_config();
    let mut test_index = TestEntityIndex::new_with_config(config)?;

    // add traits in 3 batch so that we have pending & chain items
    let ops_id = test_index.put_test_traits(0..10)?;
    test_index.wait_operations_emitted(&ops_id);
    test_index.handle_engine_events()?;
    test_index.wait_operations_committed(&ops_id[0..10]);

    let ops_id = test_index.put_test_traits(10..20)?;
    test_index.wait_operations_emitted(&ops_id);
    test_index.handle_engine_events()?;

    let ops_id = test_index.put_test_traits(20..30)?;
    test_index.wait_operations_emitted(&ops_id);
    test_index.handle_engine_events()?;

    // first page
    let query_builder = QueryBuilder::with_trait("exocore.test.TestMessage").with_count(10);
    let res = test_index.index.search(&query_builder.clone().build())?;
    let entities_id = extract_results_entities_id(&res);

    // estimated, since it may be in pending and chain store
    assert!(res.estimated_count >= 30);
    assert!(entities_id.contains(&"entity29"));
    assert!(entities_id.contains(&"entity20"));

    // second page
    let query_builder = query_builder.with_paging(res.next_page.unwrap());
    let res = test_index.index.search(&query_builder.clone().build())?;
    let entities_id = extract_results_entities_id(&res);
    assert!(entities_id.contains(&"entity19"));
    assert!(entities_id.contains(&"entity10"));

    // third page
    let query_builder = query_builder.with_paging(res.next_page.unwrap());
    let res = test_index.index.search(&query_builder.clone().build())?;
    let entities_id = extract_results_entities_id(&res);
    assert!(entities_id.contains(&"entity9"));
    assert!(entities_id.contains(&"entity0"));

    // fourth page (empty)
    let query_builder = query_builder.with_paging(res.next_page.unwrap());
    let res = test_index.index.search(&query_builder.clone().build())?;
    assert_eq!(res.entities.len(), 0);
    assert!(res.next_page.is_none());

    // test explicit after token
    let paging = Paging {
        count: 10,
        after_token: SortToken::from_u64(0).into(),
        ..Default::default()
    };
    let query_builder = query_builder.with_paging(paging);
    let res = test_index.index.search(&query_builder.clone().build())?;
    assert_eq!(res.entities.len(), 10);

    let paging = Paging {
        count: 10,
        after_token: SortToken::from_u64(std::u64::MAX).into(),
        ..Default::default()
    };
    let query_builder = query_builder.with_paging(paging);
    let res = test_index.index.search(&query_builder.clone().build())?;
    assert_eq!(res.entities.len(), 0);

    // test explicit before token
    let paging = Paging {
        count: 10,
        before_token: SortToken::from_u64(0).into(),
        ..Default::default()
    };
    let query_builder = query_builder.with_paging(paging);
    let res = test_index.index.search(&query_builder.clone().build())?;
    assert_eq!(res.entities.len(), 0);

    let paging = Paging {
        count: 10,
        before_token: SortToken::from_u64(std::u64::MAX).into(),
        ..Default::default()
    };
    let query_builder = query_builder.with_paging(paging);
    let res = test_index.index.search(&query_builder.build())?;
    assert_eq!(res.entities.len(), 10);

    Ok(())
}

#[test]
fn summary_query() -> Result<(), failure::Error> {
    let config = TestEntityIndex::create_test_config();
    let mut test_index = TestEntityIndex::new_with_config(config)?;

    let op1 = test_index.put_test_trait("entity1", "trait1", "name")?;
    let op2 = test_index.put_test_trait("entity2", "trait1", "name")?;
    test_index.wait_operations_committed(&[op1, op2]);
    test_index.handle_engine_events()?;

    let query = QueryBuilder::match_text("name").only_summary().build();
    let res = test_index.index.search(&query)?;
    assert!(res.summary);
    assert!(res.entities[0].entity.as_ref().unwrap().traits.is_empty());

    let query = QueryBuilder::match_text("name").build();
    let res = test_index.index.search(&query)?;
    assert!(!res.summary);

    let query = QueryBuilder::match_text("name")
        .only_summary_if_equals(res.hash)
        .build();
    let res = test_index.index.search(&query)?;
    assert!(res.summary);

    Ok(())
}

fn count_results_source(results: &EntityResults, source: EntityResultSource) -> usize {
    results
        .entities
        .iter()
        .filter(|r| r.source == i32::from(source))
        .count()
}

fn extract_results_entities_id(res: &EntityResults) -> Vec<&str> {
    res.entities
        .iter()
        .map(|res| res.entity.as_ref().unwrap().id.as_str())
        .collect_vec()
}

fn extract_result_messages(res: &EntityResult) -> Vec<(Trait, TestMessage)> {
    let traits = res.entity.as_ref().unwrap().traits.clone();
    traits
        .into_iter()
        .map(|trt| {
            let msg = TestMessage::decode(trt.message.as_ref().unwrap().value.as_slice()).unwrap();
            (trt, msg)
        })
        .collect()
}

fn extract_indexed_operations_id(res: MutationResults) -> Vec<OperationId> {
    res.results
        .iter()
        .map(|r| r.operation_id)
        .unique()
        .collect()
}
