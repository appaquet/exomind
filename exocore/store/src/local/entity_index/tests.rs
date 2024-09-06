use std::sync::{Arc, Mutex};

use exocore_chain::engine::Event;
use exocore_core::tests_utils::{
    assert_equal_res, assert_res, async_expect_eventually_fallible, async_test_retry,
};
use exocore_protos::{
    generated::{exocore_store::Paging, exocore_test::TestMessage},
    prost::Message,
    store::{EntityResult, EntityResultSource, EntityResults, Reference, Trait},
    test::TestMessage2,
};
use itertools::Itertools;

use crate::{
    local::{
        entity_index::test_index::TestEntityIndex, mutation_index::MutationType, EntityIndexConfig,
    },
    mutation::MutationBuilder,
    ordering::{value_from_u64, value_max},
    query::{ProjectionBuilder, QueryBuilder as Q},
};

#[tokio::test(flavor = "multi_thread")]
async fn index_full_pending_to_chain() -> anyhow::Result<()> {
    async_test_retry(|| async {
        let config = EntityIndexConfig {
            chain_index_min_depth: 1, // index when block is at depth 1 or more
            ..TestEntityIndex::test_config()
        };
        let mut test_index = TestEntityIndex::new_with_config(config).await?;
        test_index.handle_engine_events()?;

        // index a few traits, they should now be available from pending index
        let first_ops_id = test_index.put_test_traits(0..=4)?;
        test_index.wait_operations_emitted(&first_ops_id);
        test_index.handle_engine_events()?;
        let res = test_index
            .index
            .search(Q::with_trait::<TestMessage>().build())?;
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
            .search(Q::with_trait::<TestMessage>().build())?;
        let pending_res = count_results_source(&res, EntityResultSource::Pending);
        let chain_res = count_results_source(&res, EntityResultSource::Chain);
        assert_eq!(pending_res + chain_res, 10);

        // wait for second block to be committed, first operations should now be indexed
        // in chain
        test_index.wait_operations_committed(&second_ops_id);

        let test_index = Arc::new(Mutex::new(test_index)); // needed sync we pass to async FnMut
        async_expect_eventually_fallible(|| async {
            let mut test_index = test_index.lock().unwrap();
            test_index.handle_engine_events()?;

            let res = test_index
                .index
                .search(Q::with_trait::<TestMessage>().build())?;
            let pending_res = count_results_source(&res, EntityResultSource::Pending);
            let chain_res = count_results_source(&res, EntityResultSource::Chain);

            assert_equal_res(pending_res + chain_res, 10)?;
            assert_res(chain_res >= 5)?;
            Ok(())
        })
        .await?;

        Ok(())
    })
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn test_chain_index_block_depth_leeway() -> anyhow::Result<()> {
    let config = EntityIndexConfig {
        chain_index_min_depth: 1,    // index up to the depth 1 (last block in chain)
        chain_index_depth_leeway: 5, // only index once we reach depth of 5 in chain
        ..TestEntityIndex::test_config()
    };
    let mut test_index = TestEntityIndex::new_with_config(config).await?;
    test_index.handle_engine_events()?;

    let mut put_and_query = |i| -> anyhow::Result<(usize, usize)> {
        let entity_id = format!("entity{}", i);
        let trait_id = format!("trait{}", i);
        let name = format!("name{}", i);
        let op = test_index.put_test_trait(entity_id, trait_id, name)?;

        test_index.wait_operations_committed(&[op]);
        test_index.handle_engine_events()?;
        let res = test_index
            .index
            .search(Q::with_trait::<TestMessage>().build())?;

        let pending_res = count_results_source(&res, EntityResultSource::Pending);
        let chain_res = count_results_source(&res, EntityResultSource::Chain);

        Ok((pending_res, chain_res))
    };

    // first block gets indexed, but nothing gets indexed until 6th
    for i in 0..5 {
        let (pending_res, chain_res) = put_and_query(i)?;
        assert_eq!(pending_res + chain_res, i + 1, "iter {}", i);
        assert!(chain_res <= 1, "Chain {} at iter {}", chain_res, i);
    }

    // at 6th block, we expect everything to be indexed except last one
    let (pending_res, chain_res) = put_and_query(5)?;
    assert_eq!(pending_res + chain_res, 6);
    assert!(chain_res >= 5, "Chain {} at iter 6", chain_res);

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn reopen_chain_index() -> anyhow::Result<()> {
    let config = EntityIndexConfig {
        chain_index_min_depth: 0, // index as soon as new block appear
        chain_index_in_memory: false,
        ..TestEntityIndex::test_config()
    };

    // index a few traits & make sure it's in the chain index
    let mut test_index = TestEntityIndex::new_with_config(config).await?;
    let ops_id = test_index.put_test_traits(0..=9)?;
    test_index.wait_operations_committed(&ops_id);
    test_index.drain_received_events();
    test_index.index.reindex_chain()?;

    // reopen index, make sure data is still in there
    let test_index = test_index.with_restarted_node().await?;
    // traits should still be indexed
    let res = test_index
        .index
        .search(Q::with_trait::<TestMessage>().build())?;
    assert_eq!(res.entities.len(), 10);

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn reopen_chain_and_pending_transition() -> anyhow::Result<()> {
    let config = EntityIndexConfig {
        chain_index_min_depth: 2,
        chain_index_in_memory: false,
        ..TestEntityIndex::test_config()
    };

    let mut test_index = TestEntityIndex::new_with_config(config).await?;
    let query = Q::with_trait::<TestMessage>().count(100).build();

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
        test_index = test_index.with_restarted_node().await?;

        // traits should still be indexed
        let res = test_index.index.search(&query)?;
        assert_eq!(res.entities.len(), i * 10);

        range_from = range_to + 1;
    }

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn reindex_pending_on_discontinuity() -> anyhow::Result<()> {
    let mut test_index = TestEntityIndex::new().await?;

    // index traits without indexing them by clearing events
    test_index.put_test_traits(0..=5)?;
    test_index.drain_received_events();

    let res = test_index
        .index
        .search(Q::with_trait::<TestMessage>().build())?;
    assert_eq!(res.entities.len(), 0);

    // trigger discontinuity, which should force reindex
    test_index
        .index
        .handle_chain_engine_event(Event::StreamDiscontinuity)?;

    // pending is indexed
    let res = test_index
        .index
        .search(Q::with_trait::<TestMessage>().build())?;
    assert_eq!(res.entities.len(), 6);

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn chain_divergence() -> anyhow::Result<()> {
    let config = EntityIndexConfig {
        chain_index_min_depth: 0, // index as soon as new block appear
        ..TestEntityIndex::test_config()
    };
    let mut test_index = TestEntityIndex::new_with_config(config).await?;

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
        .search(Q::with_trait::<TestMessage>().build())?;
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
        .search(Q::with_trait::<TestMessage>().build())?;
    assert_eq!(res.entities.len(), 10);

    // divergence at an offset indexed in chain index will fail
    let res = test_index
        .index
        .handle_chain_engine_event(Event::ChainDiverged(0));
    assert!(res.is_err());

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn delete_entity_trait() -> anyhow::Result<()> {
    let config = EntityIndexConfig {
        chain_index_min_depth: 1, // index in chain as soon as another block is after
        ..TestEntityIndex::test_config()
    };
    let mut test_index = TestEntityIndex::new_with_config(config).await?;

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

    let pending_res = test_index
        .index
        .pending_index
        .fetch_entity_mutations("entity1")?;
    assert!(pending_res
        .mutations
        .iter()
        .any(|r| matches!(&r.mutation_type, MutationType::TraitTombstone(_))));

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn delete_all_entity_traits() -> anyhow::Result<()> {
    let config = TestEntityIndex::test_config();
    let mut test_index = TestEntityIndex::new_with_config(config).await?;

    let op1 = test_index.put_test_trait("entity1", "trait1", "name1")?;
    let op2 = test_index.put_test_trait("entity1", "trait2", "name2")?;
    test_index.wait_operations_committed(&[op1, op2]);
    test_index.handle_engine_events()?;

    let query = Q::with_id("entity1").build();
    let res = test_index.index.search(query)?;
    assert_eq!(res.entities.len(), 1);

    let op_id = test_index.delete_trait("entity1", "trait1")?;
    test_index.wait_operation_committed(op_id);
    test_index.handle_engine_events()?;

    let query = Q::with_id("entity1").build();
    let res = test_index.index.search(query)?;
    assert_eq!(res.entities.len(), 1);

    let op_id = test_index.delete_trait("entity1", "trait2")?;
    test_index.wait_operation_committed(op_id);
    test_index.handle_engine_events()?;

    let query = Q::with_id("entity1").build();
    let res = test_index.index.search(query)?;
    assert_eq!(res.entities.len(), 0);

    // if we request deleted, it should now be back
    let query = Q::with_id("entity1").include_deleted().build();
    let res = test_index.index.search(query)?;
    assert_eq!(res.entities.len(), 1);

    let entity = res.entities[0].entity.as_ref().unwrap();
    assert!(entity.deletion_date.is_some());
    assert_eq!(entity.traits.len(), 2);
    assert!(entity.traits[0].deletion_date.is_some());
    assert!(entity.traits[1].deletion_date.is_some());

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn delete_entity() -> anyhow::Result<()> {
    let config = EntityIndexConfig {
        chain_index_min_depth: 1, // index in chain as soon as another block is after
        ..TestEntityIndex::test_config()
    };
    let mut test_index = TestEntityIndex::new_with_config(config).await?;

    let op1 = test_index.put_test_trait("entity1", "trait1", "name1")?;
    let op2 = test_index.put_test_trait("entity1", "trait2", "name2")?;
    test_index.wait_operations_committed(&[op1, op2]);
    test_index.handle_engine_events()?;

    let query = Q::with_id("entity1").build();
    let res = test_index.index.search(query)?;
    assert_eq!(res.entities.len(), 1);

    let op_id = test_index.write_mutation(MutationBuilder::new().delete_entity("entity1"))?;
    test_index.wait_operation_committed(op_id);
    test_index.handle_engine_events()?;
    let query = Q::with_id("entity1").build();
    let res = test_index.index.search(query)?;
    assert_eq!(res.entities.len(), 0);

    // now bury the deletion under 1 block, which should delete for real the trait
    let op_id = test_index.put_test_trait("entity2", "trait2", "name1")?;
    test_index.wait_operation_committed(op_id);
    test_index.handle_engine_events()?;

    // should still be deleted
    let query = Q::with_id("entity1").build();
    let res = test_index.index.search(query)?;
    assert_eq!(res.entities.len(), 0);

    // if we request deleted, it should now be back
    let query = Q::with_id("entity1").include_deleted().build();
    let res = test_index.index.search(query)?;
    assert_eq!(res.entities.len(), 1);
    assert!(res.entities[0]
        .entity
        .as_ref()
        .unwrap()
        .deletion_date
        .is_some());

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn operations_deletion() -> anyhow::Result<()> {
    let config = EntityIndexConfig {
        chain_index_min_depth: 0, // index in chain as soon as a block is committed
        ..TestEntityIndex::test_config()
    };
    let mut test_index = TestEntityIndex::new_with_config(config).await?;

    let op1 = test_index.put_test_trait("entity1", "trait1", "op1")?;
    let op2 = test_index.put_test_trait("entity1", "trait1", "op2")?;
    let op3 = test_index.put_test_trait("entity1", "trait1", "op3")?;
    test_index.wait_operations_committed(&[op1, op2, op3]);
    test_index.handle_engine_events()?;

    // we have 3 mutations on same trait
    let chain_res = test_index
        .index
        .chain_index
        .fetch_entity_mutations("entity1")?;
    assert_eq!(chain_res.mutations.len(), 3);

    // mut entity has only 1 trait since all ops are on same trait
    let query = Q::with_id("entity1").build();
    let res = test_index.index.search(query)?;
    assert_eq!(res.entities.len(), 1);
    let traits_msgs = extract_result_messages(&res.entities[0]);
    assert_eq!(traits_msgs.len(), 1);

    // push operations deletion
    let op_id = test_index
        .write_mutation(MutationBuilder::new().delete_operations("entity1", vec![op1, op2, op3]))?;
    test_index.wait_operation_committed(op_id);
    test_index.handle_engine_events()?;

    // make sure deletion gets indexed by appending another op
    let op4 = test_index.put_test_trait("entity_other", "trait1", "op4")?;
    test_index.wait_operations_committed(&[op4]);
    test_index.handle_engine_events()?;

    // shouldn't be in chain index anymore
    let chain_res = test_index
        .index
        .chain_index
        .fetch_entity_mutations("entity1")?;
    assert_eq!(chain_res.mutations.len(), 0);

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn operations_deletion_marker() -> anyhow::Result<()> {
    let config = EntityIndexConfig {
        chain_index_min_depth: 1, // index in chain once we have an extra block after
        chain_index_in_memory: false,
        ..TestEntityIndex::test_config()
    };
    let mut test_index = TestEntityIndex::new_with_config(config).await?;

    let op1 = test_index.put_test_trait("entity1", "trait1", "op1")?;
    let op2 = test_index.put_test_trait("entity1", "trait1", "op2")?;
    let op3 = test_index.put_test_trait("entity1", "trait1", "op3")?;
    test_index.wait_operations_committed(&[op1, op2, op3]);
    test_index.handle_engine_events()?;

    // we have 3 mutations on same trait
    let aggr = test_index
        .index
        .fetch_aggregated_entity_mutations("entity1")?;
    assert!(!aggr.pending_deletion);
    assert_eq!(aggr.active_operations.len(), 1);

    // push operations deletion
    let op_id = test_index
        .write_mutation(MutationBuilder::new().delete_operations("entity1", vec![op1, op2, op3]))?;
    test_index.wait_operation_committed(op_id);
    test_index.handle_engine_events()?;

    // deletion marker should now be in pending index
    let aggr = test_index
        .index
        .fetch_aggregated_entity_mutations("entity1")?;
    assert!(aggr.pending_deletion);
    assert_eq!(aggr.active_operations.len(), 1);

    // push another operation, which should commit deletion in chain index
    let op4 = test_index.put_test_trait("entity_other", "trait1", "op4")?;
    test_index.wait_operations_committed(&[op4]);
    test_index.handle_engine_events()?;

    // the marker isn't in pending anymore, no active ops left
    let aggr = test_index
        .index
        .fetch_aggregated_entity_mutations("entity1")?;
    assert!(!aggr.pending_deletion);
    assert_eq!(aggr.active_operations.len(), 0);

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn query_paging() -> anyhow::Result<()> {
    let config = TestEntityIndex::test_config();
    let mut test_index = TestEntityIndex::new_with_config(config).await?;

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
    let query_builder = Q::with_trait::<TestMessage>().count(10);
    let res = test_index.index.search(query_builder.clone().build())?;
    let entities_id = extract_results_entities_id(&res);

    // estimated, since it may be in pending and chain store
    assert!(res.estimated_count >= 30);
    assert!(entities_id.contains(&"entity29"));
    assert!(entities_id.contains(&"entity20"));

    // second page
    let query_builder = query_builder.with_paging(res.next_page.unwrap());
    let res = test_index.index.search(query_builder.clone().build())?;
    let entities_id = extract_results_entities_id(&res);
    assert!(entities_id.contains(&"entity19"));
    assert!(entities_id.contains(&"entity10"));

    // third page
    let query_builder = query_builder.with_paging(res.next_page.unwrap());
    let res = test_index.index.search(query_builder.clone().build())?;
    let entities_id = extract_results_entities_id(&res);
    assert!(entities_id.contains(&"entity9"));
    assert!(entities_id.contains(&"entity0"));

    // fourth page (empty)
    let query_builder = query_builder.with_paging(res.next_page.unwrap());
    let res = test_index.index.search(query_builder.clone().build())?;
    assert_eq!(res.entities.len(), 0);
    assert!(res.next_page.is_none());

    // test explicit after token
    let paging = Paging {
        count: 10,
        after_ordering_value: Some(value_from_u64(0, 0)),
        ..Default::default()
    };
    let query_builder = query_builder.with_paging(paging);
    let res = test_index.index.search(query_builder.clone().build())?;
    assert_eq!(res.entities.len(), 10);

    let paging = Paging {
        count: 10,
        after_ordering_value: Some(value_max()),
        ..Default::default()
    };
    let query_builder = query_builder.with_paging(paging);
    let res = test_index.index.search(query_builder.clone().build())?;
    assert_eq!(res.entities.len(), 0);

    // test explicit before token
    let paging = Paging {
        count: 10,
        before_ordering_value: Some(value_from_u64(0, 0)),
        ..Default::default()
    };
    let query_builder = query_builder.with_paging(paging);
    let res = test_index.index.search(query_builder.clone().build())?;
    assert_eq!(res.entities.len(), 0);

    let paging = Paging {
        count: 10,
        before_ordering_value: Some(value_max()),
        ..Default::default()
    };
    let query_builder = query_builder.with_paging(paging);
    let res = test_index.index.search(query_builder.build())?;
    assert_eq!(res.entities.len(), 10);

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn query_multiple_mutations_paging() -> anyhow::Result<()> {
    let config = TestEntityIndex::test_config();
    let mut test_index = TestEntityIndex::new_with_config(config).await?;

    // add traits in 2 batch so that we have pending & chain items
    let ops_id = test_index.put_test_traits(0..10)?;
    test_index.wait_operations_emitted(&ops_id);
    test_index.handle_engine_events()?;
    test_index.wait_operations_committed(&ops_id[0..10]);

    let ops_id = test_index.put_test_traits(10..20)?;
    test_index.wait_operations_emitted(&ops_id);
    test_index.handle_engine_events()?;

    // override some items in first range, which will make them have 2 mutations,
    // but should only appear once in the results
    let ops_id = test_index.put_test_traits(5..7)?;
    test_index.wait_operations_emitted(&ops_id);
    test_index.handle_engine_events()?;

    // first page should contain the 2 just-modified entities
    let query_builder = Q::with_trait::<TestMessage>()
        .order_by_operations(false)
        .include_deleted()
        .count(10);
    let res = test_index.index.search(query_builder.clone().build())?;
    let page1 = extract_results_entities_id(&res);
    assert_eq!(
        &["entity6", "entity5", "entity19", "entity18"],
        &page1[0..4]
    );

    // second page shouldn't contain just-modified entities
    let query_builder = query_builder.with_paging(res.next_page.unwrap());
    let res = test_index.index.search(query_builder.build())?;
    let page2 = extract_results_entities_id(&res);
    assert_eq!(
        &["entity11", "entity10", "entity9", "entity8", "entity7", "entity4"],
        &page2[0..6]
    );
    assert!(!page2.contains(&"entity5"));
    assert!(!page2.contains(&"entity6"));

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn query_ordering() -> anyhow::Result<()> {
    let config = TestEntityIndex::test_config();
    let mut test_index = TestEntityIndex::new_with_config(config).await?;

    let ops_id = test_index.put_test_traits(0..10)?;
    test_index.wait_operations_emitted(&ops_id);
    test_index.handle_engine_events()?;
    test_index.wait_operations_committed(&ops_id[0..10]);

    // descending
    let qb = Q::matches("common").order_ascending(false);
    let res = test_index.index.search(qb.build())?;
    let ids = extract_results_entities_id(&res);
    assert_eq!(10, ids.len());
    assert_eq!("entity9", ids[0]);
    assert_eq!("entity0", ids[9]);

    // ascending
    let qb = Q::matches("common").order_ascending(true);
    let res = test_index.index.search(qb.build())?;
    let ids = extract_results_entities_id(&res);
    assert_eq!(10, ids.len());
    assert_eq!("entity0", ids[0]);
    assert_eq!("entity9", ids[9]);

    // ascending paged
    let qb = Q::matches("common").order_ascending(true).count(5);
    let res = test_index.index.search(qb.build())?;
    let ids = extract_results_entities_id(&res);
    assert_eq!(5, ids.len());
    assert_eq!("entity0", ids[0]);
    assert_eq!("entity4", ids[4]);

    let qb = Q::matches("common")
        .order_ascending(true)
        .with_paging(res.next_page.unwrap());
    let res = test_index.index.search(qb.build())?;
    let ids = extract_results_entities_id(&res);
    assert_eq!(5, ids.len());
    assert_eq!("entity5", ids[0]);
    assert_eq!("entity9", ids[4]);

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn query_boost_reference() -> anyhow::Result<()> {
    let config = TestEntityIndex::test_config();
    let mut test_index = TestEntityIndex::new_with_config(config).await?;

    let mut ops_id = Vec::new();
    for i in 0..5 {
        let ops = test_index.put_trait_message(
            format!("noref_{}", i),
            format!("trt_noref_{}", i),
            TestMessage {
                string1: "hello ".repeat(6 - i),
                ..Default::default()
            },
        )?;
        ops_id.push(ops);
    }
    for i in 0..5 {
        let ops = test_index.put_trait_message(
            format!("ref_{}", i),
            format!("trt_ref_{}", i),
            TestMessage {
                string1: "hello ".repeat(6 - i),
                ref1: Some(Reference {
                    entity_id: "inbox".into(),
                    ..Default::default()
                }),
                ..Default::default()
            },
        )?;
        ops_id.push(ops);
    }
    test_index.wait_operations_emitted(&ops_id);
    test_index.handle_engine_events()?;

    // boost reference, descending
    let qb = Q::matches("hello")
        .order_by_score(false, false, true)
        .count(10);
    let res = test_index.index.search(qb.build())?;
    let ids = extract_results_entities_id(&res);
    assert_eq!(10, ids.len());
    assert_eq!(
        vec![
            "ref_0", "ref_1", "ref_2", "ref_3", "ref_4", "noref_0", "noref_1", "noref_2",
            "noref_3", "noref_4"
        ],
        ids
    );

    // no reference boost, descending
    let qb = Q::matches("hello")
        .order_by_score(false, false, false)
        .count(10);
    let res = test_index.index.search(qb.build())?;
    let ids = extract_results_entities_id(&res);
    assert_eq!(10, ids.len());
    assert_eq!(
        vec![
            "ref_0", "noref_0", "ref_1", "noref_1", "ref_2", "noref_2", "ref_3", "noref_3",
            "ref_4", "noref_4"
        ],
        ids
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn skip_results_hash() -> anyhow::Result<()> {
    let config = TestEntityIndex::test_config();
    let mut test_index = TestEntityIndex::new_with_config(config).await?;

    let op1 = test_index.put_test_trait("entity1", "trait1", "name")?;
    let op2 = test_index.put_test_trait("entity2", "trait1", "name")?;
    test_index.wait_operations_committed(&[op1, op2]);
    test_index.handle_engine_events()?;

    let query = Q::matches("name").build();
    let res = test_index.index.search(query)?;
    assert!(!res.skipped_hash);

    let query = Q::matches("name").skip_if_results_equals(res.hash).build();
    let res = test_index.index.search(query)?;
    assert!(res.skipped_hash);

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn query_projection() -> anyhow::Result<()> {
    let config = TestEntityIndex::test_config();
    let mut test_index = TestEntityIndex::new_with_config(config).await?;

    let op1 = test_index.put_test_trait("entity1", "trait1", "name 1")?;
    let op2 = test_index.put_test_trait("entity2", "trait1", "name 2")?;
    test_index.wait_operations_committed(&[op1, op2]);
    test_index.handle_engine_events()?;

    {
        // project field #1, should return `string1`
        let proj = ProjectionBuilder::for_trait::<TestMessage>().return_fields(vec![1]);
        let query = Q::matches("name").project(proj).build();
        let res = test_index.index.search(query)?;
        let ent = res.entities[0].entity.as_ref().unwrap();
        let trt = &ent.traits[0];
        let msg = TestMessage::decode(trt.message.as_ref().unwrap().value.as_slice())?;
        assert_eq!(msg.string1, "name 2");
    }

    {
        // project field #2, should not return `string1`
        let proj = ProjectionBuilder::for_trait::<TestMessage>().return_fields(vec![2]);
        let query = Q::matches("name").project(proj).build();
        let res = test_index.index.search(query)?;
        let ent = res.entities[0].entity.as_ref().unwrap();
        let trt = &ent.traits[0];
        let msg = TestMessage::decode(trt.message.as_ref().unwrap().value.as_slice())?;
        assert!(msg.string1.is_empty());
    }

    {
        // project field on another message type, shouldn't include any traits
        let proj = ProjectionBuilder::for_trait::<TestMessage2>().return_fields(vec![2]);
        let proj_skip = ProjectionBuilder::for_all().skip();
        let query = Q::matches("name").projects(vec![proj, proj_skip]).build();
        let res = test_index.index.search(query)?;
        let ent = res.entities[0].entity.as_ref().unwrap();
        assert!(ent.traits.is_empty());
    }

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
