use std::{cell::Cell, cmp::Ordering, collections::HashMap, io::Write, iter::Peekable};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use exocore_chain::{block::Block, chain::ChainStore};
use exocore_protos::{
    generated::data_chain_capnp::chain_operation,
    prost::{Message, ProstDateTimeExt, ProstTimestampExt},
    store::{entity_mutation, CommittedEntityMutation, Entity, EntityMutation},
};
use extsort::ExternalSorter;

use super::EntityAggregator;
use crate::{
    error::Error,
    local::mutation_index::{MutationMetadata, MutationType, PutTraitMetadata},
    ordering::OrderingValueWrapper,
};

/// Iterator over entity mutations of a chain store.
///
/// The iterator first sorts all mutations by entities, blocks and operation
/// ids. Because of this, entity mutations are always returned in the same order
/// by entities, blocks and operation ids.
pub struct ChainEntityMutationIterator<'s> {
    iter: Box<dyn Iterator<Item = SortableMutation> + 's>,
}

impl<'s> ChainEntityMutationIterator<'s> {
    pub fn new<S: ChainStore>(
        chain_store: &'s S,
    ) -> Result<ChainEntityMutationIterator<'s>, Error> {
        let mut error: Option<Error> = None;
        let has_error = Cell::new(false);
        let mutations = chain_store
            .blocks_iter(0)
            .take_while(|_| !has_error.get())
            .flat_map(|block| match extract_block_mutations(block) {
                Ok(mutations) => mutations,
                Err(err) => {
                    error = Some(err);
                    has_error.set(true);
                    vec![]
                }
            });

        let sorter = ExternalSorter::new();
        let sorted_mutations = sorter
            .sort_by(mutations, SortableMutation::compare)
            .map_err(|err| Error::Other(anyhow!("couldn't sort mutations: {}", err)))?;

        // since the sorter goes through all mutations, any error found decoding
        // mutations will be found at this point
        if let Some(err) = error {
            return Err(err);
        }

        Ok(ChainEntityMutationIterator {
            iter: Box::new(sorted_mutations),
        })
    }
}

fn extract_block_mutations(
    block: Result<
        exocore_chain::block::DataBlock<exocore_chain::chain::ChainData>,
        exocore_chain::chain::Error,
    >,
) -> Result<Vec<SortableMutation>, Error> {
    let block = block?;

    let mut mutations = Vec::new();
    for operation in block.operations_iter()? {
        let operation_reader = operation.get_reader()?;

        // only export entry operations (actual data, not chain maintenance related
        // operations)
        let data = match operation_reader
            .get_operation()
            .which()
            .map_err(|err| Error::Serialization(err.into()))?
        {
            chain_operation::operation::Entry(Ok(entry)) => entry.get_data()?,
            _ => continue,
        };

        mutations.push(SortableMutation {
            inner: CommittedEntityMutation {
                block_offset: block.offset(),
                operation_id: operation_reader.get_operation_id(),
                mutation: Some(EntityMutation::decode(data)?),
            },
        });
    }

    Ok(mutations)
}

impl<'s> Iterator for ChainEntityMutationIterator<'s> {
    type Item = CommittedEntityMutation;

    fn next(&mut self) -> Option<Self::Item> {
        let mutation = self.iter.next()?;
        Some(mutation.inner)
    }
}

/// Iterator over the entities of the chain.
pub struct ChainEntityIterator<'s> {
    mutations: Peekable<ChainEntityMutationIterator<'s>>,
    buffer: Vec<MutationMetadata>,
}

impl<'s> ChainEntityIterator<'s> {
    pub fn new<S: ChainStore>(chain_store: &'s S) -> Result<ChainEntityIterator<'s>, Error> {
        Ok(ChainEntityIterator {
            mutations: ChainEntityMutationIterator::new(chain_store)?.peekable(),
            buffer: Vec::new(),
        })
    }

    fn extract_next_entity(&mut self) -> Result<Option<Entity>, Error> {
        self.buffer.clear();

        let mut entity_id = String::new();
        let mut traits = HashMap::new();
        while let Some(next) = self.mutations.peek() {
            let next_id = &next
                .mutation
                .as_ref()
                .expect("entity mutation didn't have a mutation")
                .entity_id;
            if entity_id.is_empty() {
                entity_id.clone_from(next_id);
            } else if entity_id != *next_id {
                break;
            }

            let mutation = self
                .mutations
                .next()
                .expect("had a peek, but couldn't get next");
            let op_id = mutation.operation_id;

            if let Some(metadata) = entity_to_mutation_metadata(&mutation)? {
                self.buffer.push(metadata);

                if let Some(entity_mutation::Mutation::PutTrait(put)) =
                    mutation.mutation.and_then(|em| em.mutation)
                {
                    if let Some(trt) = put.r#trait {
                        traits.insert(op_id, trt);
                    }
                }
            }
        }

        if entity_id.is_empty() {
            return Ok(None);
        }

        let aggr = EntityAggregator::new(self.buffer.drain(..));
        let mut entity = Entity {
            id: entity_id,
            traits: vec![],
            creation_date: aggr.creation_date.map(|d| d.to_proto_timestamp()),
            modification_date: aggr.modification_date.map(|d| d.to_proto_timestamp()),
            deletion_date: aggr.deletion_date.map(|d| d.to_proto_timestamp()),
            last_operation_id: aggr.last_operation_id,
        };

        for (_trait_id, trait_aggr) in aggr.traits {
            let Some((mut_meta, _put_mut)) = trait_aggr.last_put_mutation() else {
                continue;
            };

            if let Some(mut trt) = traits.remove(&mut_meta.operation_id) {
                trt.creation_date = trait_aggr.creation_date.map(|d| d.to_proto_timestamp());
                trt.modification_date =
                    trait_aggr.modification_date.map(|d| d.to_proto_timestamp());
                trt.deletion_date = trait_aggr.deletion_date.map(|d| d.to_proto_timestamp());
                trt.last_operation_id = trait_aggr.last_operation_id.unwrap_or_default();
                entity.traits.push(trt);
            }
        }

        Ok(Some(entity))
    }
}

impl<'s> Iterator for ChainEntityIterator<'s> {
    type Item = Result<Entity, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.extract_next_entity().transpose()
    }
}

fn entity_to_mutation_metadata(
    committed_entity: &CommittedEntityMutation,
) -> Result<Option<MutationMetadata>, Error> {
    use exocore_protos::store::entity_mutation::Mutation;
    let mutation = committed_entity
        .mutation
        .as_ref()
        .ok_or_else(|| Error::Other(anyhow!("no entity mutation")))?;

    let mutation_type = mutation
        .mutation
        .as_ref()
        .ok_or_else(|| Error::Other(anyhow!("no entity mutation")))?;

    let metadata = match mutation_type {
        Mutation::PutTrait(put) => Some(put_trait_to_metadata(put, committed_entity, mutation)?),
        Mutation::DeleteTrait(del) => Some(del_trait_to_metadata(committed_entity, mutation, del)),
        Mutation::DeleteEntity(del) => {
            Some(del_entity_to_metadata(committed_entity, mutation, del))
        }
        _ => None,
    };

    Ok(metadata)
}

fn put_trait_to_metadata(
    put: &exocore_protos::store::PutTraitMutation,
    committed_entity: &CommittedEntityMutation,
    mutation: &EntityMutation,
) -> Result<MutationMetadata, Error> {
    let trt = put
        .r#trait
        .as_ref()
        .ok_or_else(|| Error::Other(anyhow!("no trait in PutTrait mutation")))?;
    Ok(MutationMetadata {
        operation_id: committed_entity.operation_id,
        block_offset: Some(committed_entity.block_offset),
        entity_id: mutation.entity_id.clone(),
        mutation_type: MutationType::TraitPut(PutTraitMetadata {
            trait_id: trt.id.clone(),
            trait_type: None,
            creation_date: trt.creation_date.as_ref().map(|d| d.to_chrono_datetime()),
            modification_date: trt
                .modification_date
                .as_ref()
                .map(|d| d.to_chrono_datetime()),
            has_reference: false,
        }),
        sort_value: OrderingValueWrapper::default(),
    })
}

fn del_trait_to_metadata(
    committed_entity: &CommittedEntityMutation,
    mutation: &EntityMutation,
    del: &exocore_protos::store::DeleteTraitMutation,
) -> MutationMetadata {
    MutationMetadata {
        operation_id: committed_entity.operation_id,
        block_offset: Some(committed_entity.block_offset),
        entity_id: mutation.entity_id.clone(),
        mutation_type: MutationType::TraitTombstone(del.trait_id.clone()),
        sort_value: OrderingValueWrapper::default(),
    }
}

fn del_entity_to_metadata(
    committed_entity: &CommittedEntityMutation,
    mutation: &EntityMutation,
    _del: &exocore_protos::store::DeleteEntityMutation,
) -> MutationMetadata {
    MutationMetadata {
        operation_id: committed_entity.operation_id,
        block_offset: Some(committed_entity.block_offset),
        entity_id: mutation.entity_id.clone(),
        mutation_type: MutationType::EntityTombstone,
        sort_value: OrderingValueWrapper::default(),
    }
}

/// Entity mutation wrapper used for external sorting.
///
/// External sorting needs to be able to serialize and deserialize
/// since it uses disk-based storage when sorted data wouldn't fit
/// in memory.
struct SortableMutation {
    inner: CommittedEntityMutation,
}

impl extsort::Sortable for SortableMutation {
    fn encode<W: Write>(&self, writer: &mut W) {
        let mutation = self.inner.encode_to_vec();
        writer
            .write_u64::<LittleEndian>(mutation.len() as u64)
            .unwrap();
        writer.write_all(&mutation).unwrap();
    }

    fn decode<R: std::io::Read>(reader: &mut R) -> Option<Self> {
        let len = reader.read_u64::<LittleEndian>().ok()?; // interpret failure as end of stream
        let mut data = vec![0; len as usize];
        reader.read_exact(&mut data).unwrap();

        Some(SortableMutation {
            inner: CommittedEntityMutation::decode(data.as_ref()).unwrap(),
        })
    }
}

impl SortableMutation {
    fn compare(a: &SortableMutation, b: &SortableMutation) -> Ordering {
        let a_entity = a.inner.mutation.as_ref().unwrap();
        let b_entity = b.inner.mutation.as_ref().unwrap();

        match a_entity.entity_id.cmp(&b_entity.entity_id) {
            Ordering::Less => return Ordering::Less,
            Ordering::Equal => (),
            Ordering::Greater => return Ordering::Greater,
        }

        match a.inner.block_offset.cmp(&b.inner.block_offset) {
            Ordering::Less => return Ordering::Less,
            Ordering::Equal => (),
            Ordering::Greater => return Ordering::Greater,
        }

        a.inner.operation_id.cmp(&b.inner.operation_id)
    }
}

#[cfg(test)]
mod tests {
    use exocore_protos::store::Trait;

    use super::*;
    use crate::local::entity_index::test_index::TestEntityIndex;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_entity_iterator() -> anyhow::Result<()> {
        let config = TestEntityIndex::test_config();
        let mut ti = TestEntityIndex::new_with_config(config).await?;

        let ops = vec![
            ti.put_test_trait("entity1", "trait1", "data")?,
            ti.put_test_trait("entity1", "trait2", "data")?,
            ti.put_test_trait("entity1", "trait3", "data")?,
            ti.put_test_trait("entity2", "trait1", "data")?,
            ti.delete_trait("entity1", "trait2")?,
            ti.put_test_trait("entity3", "trait1", "data")?,
            ti.delete_entity("entity3")?,
        ];

        ti.wait_operations_committed(&ops);

        // restart node to get access to its store (since engine takes ownership on
        // start)
        ti.cluster.stop_node(0);
        ti.cluster.create_node(0)?;

        let chain_store = ti.cluster.chain_stores[0].as_ref().unwrap();
        let iter = ChainEntityIterator::new(chain_store).unwrap();
        let entities = iter.collect::<Result<Vec<Entity>, Error>>()?;

        assert_eq!(entities.len(), 3);
        assert_eq!(entities[0].id, "entity1"); // they are sorted by id by the iterator
        assert_eq!(entities[1].id, "entity2");
        assert_eq!(entities[2].id, "entity3");

        assert_eq!(entities[0].traits.len(), 3);
        assert!(find_trait(&entities[0].traits, "trait1")
            .deletion_date
            .is_none());
        assert!(find_trait(&entities[0].traits, "trait2")
            .deletion_date
            .is_some()); // trait2 got deleted

        assert!(entities[2].deletion_date.is_some());

        Ok(())
    }

    fn find_trait<'t>(traits: &'t [Trait], id: &str) -> &'t Trait {
        traits.iter().find(|trt| trt.id == id).unwrap()
    }
}
