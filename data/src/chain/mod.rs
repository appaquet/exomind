use std::ops::Range;

use exocore_common::data_chain_capnp::{
    block, block_operation_header, block_signature, block_signatures,
};
use exocore_common::node::{Node, NodeID, Nodes};
use exocore_common::security::hash::{Multihash, Sha3Hasher, StreamHasher};
use exocore_common::security::signature::Signature;
use exocore_common::serialization::framed::{
    FrameBuilder, OwnedFrame, OwnedTypedFrame, SignedFrame, TypedFrame, TypedSliceFrame,
};
use exocore_common::serialization::protos::data_chain_capnp::pending_operation;
use exocore_common::serialization::protos::OperationID;
use exocore_common::serialization::{capnp, framed};

pub type BlockOffset = u64;
pub type BlockDepth = u64;
pub type BlockSignaturesSize = u16;

pub mod directory;

///
/// Persistence for the chain
///
pub trait ChainStore: Send + Sync + 'static {
    fn segments(&self) -> Vec<Segment>;

    fn write_block<B: Block>(&mut self, block: &B) -> Result<BlockOffset, Error>;

    fn blocks_iter(&self, from_offset: BlockOffset) -> Result<StoredBlockIterator, Error>;

    fn blocks_iter_reverse(
        &self,
        from_next_offset: BlockOffset,
    ) -> Result<StoredBlockIterator, Error>;

    fn get_block(&self, offset: BlockOffset) -> Result<BlockRef, Error>;

    fn get_block_from_next_offset(&self, next_offset: BlockOffset) -> Result<BlockRef, Error>;

    fn get_last_block(&self) -> Result<Option<BlockRef>, Error>;

    fn get_block_by_operation_id(
        &self,
        operation_id: OperationID,
    ) -> Result<Option<BlockRef>, Error>;

    fn truncate_from_offset(&mut self, offset: BlockOffset) -> Result<(), Error>;
}

///
/// Segment of the chain with a specified offsets range, in bytes.
///
/// The upper range is exclusive. You can use `get_block_from_next_offset` to get the last block
/// of the segment.
///
#[derive(Clone, Debug, PartialEq)]
pub struct Segment {
    pub range: Range<BlockOffset>,
}

///
/// Iterator over stored blocks.
///
type StoredBlockIterator<'pers> = Box<dyn Iterator<Item = BlockRef<'pers>> + 'pers>;

///
/// A trait representing a block stored or to be stored in the chain.
/// It can either be a referenced block (`BlockRef`) or a in-memory block (`BlockOwned`).
///
/// A block consists of 3 parts:
///  * Block header
///  * Operations' bytes (capnp serialized `pending_operation` frames)
///  * Block signatures
///
/// The block header and operations' data are the same on all nodes. Since a node writes a block
/// as soon as it has enough signatures, signatures can differ from one node to the other. Signatures
/// frame is pre-allocated, which means that not all signatures may fit. But in theory, it should always
/// contain enough space for all nodes to add their own signature.
///
pub trait Block {
    type BlockType: TypedFrame<block::Owned> + SignedFrame;
    type SignaturesType: TypedFrame<block_signatures::Owned> + SignedFrame;

    fn offset(&self) -> BlockOffset;
    fn block(&self) -> &Self::BlockType;
    fn operations_data(&self) -> &[u8];
    fn signatures(&self) -> &Self::SignaturesType;

    #[inline]
    fn total_size(&self) -> usize {
        self.block().frame_size() + self.operations_data().len() + self.signatures().frame_size()
    }

    #[inline]
    fn next_offset(&self) -> BlockOffset {
        self.offset() + self.total_size() as BlockOffset
    }

    #[inline]
    fn copy_data_into(&self, data: &mut [u8]) {
        let operations_data = self.operations_data();
        let operations_offset = self.block().frame_size();
        let signatures_offset = operations_offset + operations_data.len();

        self.block().copy_into(data);
        (&mut data[operations_offset..signatures_offset]).copy_from_slice(operations_data);
        self.signatures().copy_into(&mut data[signatures_offset..]);
    }

    fn as_data_vec(&self) -> Vec<u8> {
        vec![
            self.block().frame_data(),
            self.operations_data(),
            self.signatures().frame_data(),
        ]
        .concat()
    }

    fn operations_iter(&self) -> Result<BlockOperationsIterator, Error> {
        let block_reader: block::Reader = self.block().get_typed_reader()?;
        let operations_header = block_reader
            .get_operations_header()?
            .iter()
            .map(|reader| BlockOperationHeader::from_reader(&reader))
            .collect::<Vec<_>>();

        Ok(BlockOperationsIterator {
            index: 0,
            operations_header,
            operations_data: self.operations_data(),
            last_error: None,
        })
    }

    fn get_operation(
        &self,
        operation_id: OperationID,
    ) -> Result<Option<TypedSliceFrame<pending_operation::Owned>>, Error> {
        // TODO: Implement binary search in operations, since they are sorted: https://github.com/appaquet/exocore/issues/43
        let operation = self.operations_iter()?.find(|operation| {
            if let Ok(operation_reader) = operation.get_typed_reader() {
                operation_reader.get_operation_id() == operation_id
            } else {
                false
            }
        });

        Ok(operation)
    }
}

///
/// Iterator over operations stored in a block.
///
pub struct BlockOperationsIterator<'a> {
    index: usize,
    operations_header: Vec<BlockOperationHeader>,
    operations_data: &'a [u8],
    last_error: Option<Error>,
}

impl<'a> Iterator for BlockOperationsIterator<'a> {
    type Item = TypedSliceFrame<'a, pending_operation::Owned>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.operations_header.len() {
            return None;
        }

        let header = &self.operations_header[self.index];
        self.index += 1;

        let offset_from = header.data_offset as usize;
        let offset_to = header.data_offset as usize + header.data_size as usize;

        let frame_res = TypedSliceFrame::new(&self.operations_data[offset_from..offset_to]);
        match frame_res {
            Ok(frame) => Some(frame),
            Err(err) => {
                self.last_error = Some(err.into());
                None
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.index, Some(self.operations_data.len()))
    }
}

///
/// In-memory block.
///
pub struct BlockOwned {
    pub offset: BlockOffset,
    pub block: OwnedTypedFrame<block::Owned>,
    pub operations_data: Vec<u8>,
    pub signatures: OwnedTypedFrame<block_signatures::Owned>,
}

impl BlockOwned {
    pub fn new(
        offset: BlockOffset,
        block: OwnedTypedFrame<block::Owned>,
        operations_data: Vec<u8>,
        signatures: OwnedTypedFrame<block_signatures::Owned>,
    ) -> BlockOwned {
        BlockOwned {
            offset,
            block,
            operations_data,
            signatures,
        }
    }

    pub fn new_genesis(nodes: &Nodes, node: &Node) -> Result<BlockOwned, Error> {
        let operations = BlockOperations::empty();
        let block = Self::new_with_prev_info(nodes, node, 0, 0, 0, &[], 0, operations)?;

        // TODO: Add master signature after doing https://github.com/appaquet/exocore/issues/46

        Ok(block)
    }

    pub fn new_with_prev_block<B>(
        nodes: &Nodes,
        node: &Node,
        previous_block: &B,
        proposed_operation_id: u64,
        operations: BlockOperations,
    ) -> Result<BlockOwned, Error>
    where
        B: Block,
    {
        let previous_block_reader = previous_block.block().get_typed_reader()?;

        let previous_offset = previous_block.offset();
        let previous_hash = previous_block
            .block()
            .signature_data()
            .expect("Previous block didn't have a signature");

        let offset = previous_block.next_offset();
        let depth = previous_block_reader.get_depth();

        Self::new_with_prev_info(
            nodes,
            node,
            offset,
            depth,
            previous_offset,
            previous_hash,
            proposed_operation_id,
            operations,
        )
    }

    // TODO: Should be fixed once we do https://github.com/appaquet/exocore/issues/37
    #[allow(clippy::too_many_arguments)]
    pub fn new_with_prev_info(
        nodes: &Nodes,
        node: &Node,
        offset: BlockOffset,
        depth: BlockDepth,
        previous_offset: BlockOffset,
        previous_hash: &[u8],
        proposed_operation_id: u64,
        operations: BlockOperations,
    ) -> Result<BlockOwned, Error> {
        let operations_data_size = operations.data.len() as u32;

        // initialize block
        let mut block_frame_builder = FrameBuilder::<block::Owned>::new();
        let mut block_builder: block::Builder = block_frame_builder.get_builder_typed();
        block_builder.set_offset(offset);
        block_builder.set_depth(depth);
        block_builder.set_previous_offset(previous_offset);
        block_builder.set_previous_hash(previous_hash);
        block_builder.set_proposed_operation_id(proposed_operation_id);
        block_builder.set_proposed_node_id(&node.id());
        block_builder.set_operations_size(operations_data_size);
        block_builder.set_operations_hash(&operations.multihash_bytes);

        let mut operations_builder = block_builder
            .reborrow()
            .init_operations_header(operations.headers.len() as u32);
        for (i, header_builder) in operations.headers.iter().enumerate() {
            let mut entry_builder = operations_builder.reborrow().get(i as u32);
            header_builder.copy_into_builder(&mut entry_builder);
        }

        // create an empty signature for each node as a placeholder to find the size required for signatures
        let mut signature_frame_builder =
            BlockSignatures::empty_signatures_for_nodes(nodes).to_frame_builder();
        let mut signature_builder = signature_frame_builder.get_builder_typed();
        signature_builder.set_operations_size(operations_data_size);
        let signature_frame = signature_frame_builder.as_owned_framed(node.frame_signer())?;

        // set required signatures size in block
        block_builder.set_signatures_size(signature_frame.frame_size() as u16);
        let block_frame = block_frame_builder.as_owned_framed(node.frame_signer())?;

        Ok(BlockOwned {
            offset,
            block: block_frame,
            operations_data: operations.data,
            signatures: signature_frame,
        })
    }
}

impl Block for BlockOwned {
    type BlockType = framed::OwnedTypedFrame<block::Owned>;
    type SignaturesType = framed::OwnedTypedFrame<block_signatures::Owned>;

    #[inline]
    fn offset(&self) -> u64 {
        self.offset
    }

    #[inline]
    fn block(&self) -> &Self::BlockType {
        &self.block
    }

    #[inline]
    fn operations_data(&self) -> &[u8] {
        &self.operations_data
    }

    #[inline]
    fn signatures(&self) -> &Self::SignaturesType {
        &self.signatures
    }
}

///
/// A referenced block
///
pub struct BlockRef<'a> {
    pub offset: BlockOffset,
    pub block: framed::TypedSliceFrame<'a, block::Owned>,
    pub operations_data: &'a [u8],
    pub signatures: framed::TypedSliceFrame<'a, block_signatures::Owned>,
}

impl<'a> BlockRef<'a> {
    pub fn new(data: &[u8]) -> Result<BlockRef, Error> {
        let block = framed::TypedSliceFrame::new(data)?;
        let block_reader: block::Reader = block.get_typed_reader()?;

        let operations_offset = block.frame_size();
        let operations_size = block_reader.get_operations_size() as usize;
        let signatures_offset = operations_offset + operations_size;

        if signatures_offset >= data.len() {
            return Err(Error::OutOfBound(format!(
                "Signature offset {} is after data len {}",
                signatures_offset,
                data.len()
            )));
        }

        let operations_data = &data[operations_offset..operations_offset + operations_size];
        let signatures = framed::TypedSliceFrame::new(&data[signatures_offset..])?;

        Ok(BlockRef {
            offset: block_reader.get_offset(),
            block,
            operations_data,
            signatures,
        })
    }
}

impl<'a> Block for BlockRef<'a> {
    type BlockType = framed::TypedSliceFrame<'a, block::Owned>;
    type SignaturesType = framed::TypedSliceFrame<'a, block_signatures::Owned>;

    #[inline]
    fn offset(&self) -> u64 {
        self.offset
    }

    #[inline]
    fn block(&self) -> &Self::BlockType {
        &self.block
    }

    #[inline]
    fn operations_data(&self) -> &[u8] {
        &self.operations_data
    }

    #[inline]
    fn signatures(&self) -> &Self::SignaturesType {
        &self.signatures
    }
}

///
/// Wraps operations header stored in a block.
///
pub struct BlockOperations {
    multihash_bytes: Vec<u8>,
    headers: Vec<BlockOperationHeader>,
    data: Vec<u8>,
}

impl BlockOperations {
    fn empty() -> BlockOperations {
        BlockOperations {
            multihash_bytes: Vec::new(),
            headers: Vec::new(),
            data: Vec::new(),
        }
    }

    pub fn from_operations<I, F>(sorted_operations: I) -> Result<BlockOperations, Error>
    where
        I: Iterator<Item = F>,
        F: TypedFrame<pending_operation::Owned>,
    {
        let mut hasher = Sha3Hasher::new_256();
        let mut headers = Vec::new();
        let mut data = Vec::new();

        for operation in sorted_operations {
            let operation_reader = operation.get_typed_reader()?;
            let offset = data.len();
            let entry_data = operation.frame_data();
            hasher.consume_signed_frame(&operation);
            data.extend_from_slice(entry_data);

            headers.push(BlockOperationHeader {
                operation_id: operation_reader.get_operation_id(),
                data_offset: offset as u32,
                data_size: (data.len() - offset) as u32,
            });
        }

        Ok(BlockOperations {
            multihash_bytes: hasher.into_multihash_bytes(),
            headers,
            data,
        })
    }

    pub fn hash_operations<I, F>(sorted_operations: I) -> Result<Multihash, Error>
    where
        I: Iterator<Item = F>,
        F: TypedFrame<pending_operation::Owned>,
    {
        let mut hasher = Sha3Hasher::new_256();
        for operation in sorted_operations {
            hasher.consume_signed_frame(&operation);
        }
        Ok(hasher.into_multihash())
    }

    pub fn multihash_bytes(&self) -> &[u8] {
        &self.multihash_bytes
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }
}

///
/// Header of an operation stored within a block. It represents the position in the bytes of the block.
///
struct BlockOperationHeader {
    operation_id: u64,
    data_offset: u32,
    data_size: u32,
}

impl BlockOperationHeader {
    fn from_reader(reader: &block_operation_header::Reader) -> BlockOperationHeader {
        BlockOperationHeader {
            operation_id: reader.get_operation_id(),
            data_offset: reader.get_data_offset(),
            data_size: reader.get_data_size(),
        }
    }

    fn copy_into_builder(&self, builder: &mut block_operation_header::Builder) {
        builder.set_operation_id(self.operation_id);
        builder.set_data_size(self.data_size);
        builder.set_data_offset(self.data_offset);
    }
}

///
/// Represents signatures stored in a block. Since a node writes a block as soon as it has enough signatures, signatures can
/// differ from one node to the other. Signatures frame is pre-allocated, which means that not all signatures may fit. But in
/// theory, it should always contain enough space for all nodes to add their own signature.
///
pub struct BlockSignatures {
    signatures: Vec<BlockSignature>,
}

impl BlockSignatures {
    pub fn new_from_signatures(signatures: Vec<BlockSignature>) -> BlockSignatures {
        BlockSignatures { signatures }
    }

    pub fn empty_signatures_for_nodes(nodes: &Nodes) -> BlockSignatures {
        let signatures = nodes
            .nodes()
            .map(|node| BlockSignature {
                node_id: node.id().clone(),
                signature: Signature::empty(),
            })
            .collect();

        BlockSignatures { signatures }
    }

    pub fn to_frame_builder(&self) -> FrameBuilder<block_signatures::Owned> {
        let mut frame_builder = FrameBuilder::new();

        let signatures_builder: block_signatures::Builder = frame_builder.get_builder_typed();
        let mut signatures_array = signatures_builder.init_signatures(self.signatures.len() as u32);
        for (i, signature) in self.signatures.iter().enumerate() {
            let mut signature_builder = signatures_array.reborrow().get(i as u32);
            signature.copy_into_builder(&mut signature_builder);
        }

        frame_builder
    }

    pub fn to_frame_for_existing_block(
        &self,
        node: &Node,
        block_reader: &block::Reader,
    ) -> Result<OwnedTypedFrame<block_signatures::Owned>, Error> {
        let expected_signatures_size = usize::from(block_reader.get_signatures_size());

        let mut signatures_frame_builder = self.to_frame_builder();
        let mut signatures_builder: block_signatures::Builder =
            signatures_frame_builder.get_builder_typed();
        signatures_builder.set_operations_size(block_reader.get_operations_size());
        let signatures_frame = signatures_frame_builder.as_owned_framed(node.frame_signer())?;

        // make sure that the signatures frame size is not higher than pre-allocated space in block
        if signatures_frame.frame_size() > expected_signatures_size {
            return Err(Error::Integrity(format!(
                "Block local signatures are taking more space than allocated space ({} > {})",
                signatures_frame.frame_size(),
                block_reader.get_signatures_size()
            )));
        }

        // build a signatures frame that has the right amount of space as defined at the block
        let mut signatures_data = signatures_frame.frame_data().to_vec();
        while signatures_data.len() != expected_signatures_size {
            signatures_data.push(0);
        }
        let signatures_frame_padded = OwnedFrame::new(signatures_data)?.into_typed();

        Ok(signatures_frame_padded)
    }
}

///
/// Represents a signature of the block by one node, using its own key to sign the block's hash.
///
pub struct BlockSignature {
    pub node_id: NodeID,
    pub signature: Signature,
}

impl BlockSignature {
    pub fn new(node_id: NodeID, signature: Signature) -> BlockSignature {
        BlockSignature { node_id, signature }
    }

    pub fn copy_into_builder(&self, builder: &mut block_signature::Builder) {
        builder.set_node_id(&self.node_id);
        builder.set_node_signature(self.signature.get_bytes());
    }
}

///
/// Chain related errors
///
#[derive(Debug, Clone, PartialEq, Fail)]
pub enum Error {
    #[fail(display = "The store is in an unexpected state: {}", _0)]
    UnexpectedState(String),
    #[fail(display = "Error from the framing serialization: {:?}", _0)]
    Framing(#[fail(cause)] framed::Error),
    #[fail(display = "The store has an integrity problem: {}", _0)]
    Integrity(String),
    #[fail(display = "A segment has reached its full capacity")]
    SegmentFull,
    #[fail(display = "An offset is out of the chain data: {}", _0)]
    OutOfBound(String),
    #[fail(display = "IO error of kind {:?}: {}", _0, _1)]
    IO(std::io::ErrorKind, String),
    #[fail(display = "Field is not in capnp schema: code={}", _0)]
    SerializationNotInSchema(u16),
    #[fail(display = "Error in capnp serialization: kind={:?} msg={}", _0, _1)]
    Serialization(capnp::ErrorKind, String),
    #[fail(display = "An error occurred: {}", _0)]
    Other(String),
}

impl Error {
    pub fn is_fatal(&self) -> bool {
        match self {
            Error::UnexpectedState(_) | Error::Integrity(_) | Error::IO(_, _) => true,
            _ => false,
        }
    }
}

impl From<framed::Error> for Error {
    fn from(err: framed::Error) -> Self {
        Error::Framing(err)
    }
}

impl From<capnp::NotInSchema> for Error {
    fn from(err: capnp::NotInSchema) -> Self {
        Error::SerializationNotInSchema(err.0)
    }
}

impl From<capnp::Error> for Error {
    fn from(err: capnp::Error) -> Self {
        Error::Serialization(err.kind, err.description)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pending::PendingOperation;

    #[test]
    fn test_block_create_and_read() -> Result<(), failure::Error> {
        let mut nodes = Nodes::new();
        let node1 = Node::new("node1".to_string());
        nodes.add(node1.clone());

        let first_block = BlockOwned::new_genesis(&nodes, &node1)?;

        let operations = vec![PendingOperation::new_entry(123, "node1", b"some_data")
            .as_owned_framed(node1.frame_signer())?];
        let operations = BlockOperations::from_operations(operations.into_iter())?;

        let second_block =
            BlockOwned::new_with_prev_block(&nodes, &node1, &first_block, 0, operations)?;

        let mut data = [0u8; 5000];
        second_block.copy_data_into(&mut data);

        let read_second_block = BlockRef::new(&data[0..second_block.total_size()])?;
        assert_eq!(
            second_block.block.frame_data(),
            read_second_block.block.frame_data()
        );
        assert_eq!(
            second_block.operations_data,
            read_second_block.operations_data
        );
        assert_eq!(
            second_block.signatures.frame_data(),
            read_second_block.signatures.frame_data()
        );

        let block_reader: block::Reader = second_block.block.get_typed_reader()?;
        assert_eq!(block_reader.get_offset(), first_block.next_offset());
        assert_eq!(
            block_reader.get_signatures_size(),
            second_block.signatures.frame_size() as u16
        );
        assert_eq!(
            block_reader.get_operations_size(),
            second_block.operations_data.len() as u32
        );

        let signatures_reader: block_signatures::Reader =
            second_block.signatures.get_typed_reader()?;
        assert_eq!(
            signatures_reader.get_operations_size(),
            second_block.operations_data.len() as u32
        );

        let signatures = signatures_reader.get_signatures()?;
        assert_eq!(signatures.len(), 1);

        Ok(())
    }

    #[test]
    fn test_block_operations() -> Result<(), failure::Error> {
        let mut nodes = Nodes::new();
        let node1 = Node::new("node1".to_string());
        nodes.add(node1.clone());
        let genesis = BlockOwned::new_genesis(&nodes, &node1)?;

        // 0 operations
        let block =
            BlockOwned::new_with_prev_block(&nodes, &node1, &genesis, 0, BlockOperations::empty())?;
        assert_eq!(block.operations_iter()?.count(), 0);

        // 5 operations
        let operations = (0..5)
            .map(|i| {
                PendingOperation::new_entry(i, "node1", b"op1")
                    .as_owned_framed(node1.frame_signer())
                    .unwrap()
            })
            .collect::<Vec<_>>();

        let block_operations = BlockOperations::from_operations(operations.into_iter())?;
        let block = BlockOwned::new_with_prev_block(&nodes, &node1, &genesis, 0, block_operations)?;
        assert_eq!(block.operations_iter()?.count(), 5);

        Ok(())
    }
}
