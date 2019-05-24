use crate::operation::OperationId;
use exocore_common::cell::{Cell, FullCell};
use exocore_common::crypto::hash::{Digest, Multihash, MultihashDigest, Sha3_256};
use exocore_common::crypto::signature::Signature;
use exocore_common::data_chain_capnp::{
    block, block_operation_header, block_signature, block_signatures,
};
use exocore_common::node::NodeId;
use exocore_common::serialization::framed::{
    FrameBuilder, OwnedTypedFrame, SignedFrame, TypedFrame, TypedSliceFrame,
};
use exocore_common::serialization::protos::data_chain_capnp::pending_operation;
use exocore_common::serialization::{capnp, framed};

pub type BlockOffset = u64;
pub type BlockDepth = u64;
pub type BlockOperationsSize = u32;
pub type BlockSignaturesSize = u16;

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

    fn to_owned(&self) -> BlockOwned {
        BlockOwned {
            offset: self.offset(),
            block: self.block().to_owned(),
            operations_data: self.operations_data().to_vec(),
            signatures: self.signatures().to_owned(),
        }
    }

    fn get_depth(&self) -> Result<BlockDepth, Error> {
        let reader = self.block().get_typed_reader()?;
        Ok(reader.get_depth())
    }

    fn get_proposed_operation_id(&self) -> Result<OperationId, Error> {
        let reader = self.block().get_typed_reader()?;
        Ok(reader.get_proposed_operation_id())
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
        operation_id: OperationId,
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

    fn validate(&self) -> Result<(), Error> {
        let block_reader: block::Reader = self.block().get_typed_reader()?;

        let sig_size_header = block_reader.get_signatures_size() as usize;
        let sig_size_stored = self.signatures().frame_size();
        if sig_size_header != sig_size_stored {
            return Err(Error::Integrity(format!(
                "Signatures size don't match: sig_size_header={}, sig_size_stored={}",
                sig_size_header, sig_size_stored
            )));
        }

        let ops_size_header = block_reader.get_operations_size() as usize;
        let ops_size_stored = self.operations_data().len();
        if ops_size_header != ops_size_stored {
            return Err(Error::Integrity(format!(
                "Operations size don't match: ops_size_header={}, ops_size_stored={}",
                ops_size_header, ops_size_stored
            )));
        }

        if ops_size_header > 0 {
            let ops_hash_stored =
                BlockOperations::hash_operations(self.operations_iter()?)?.into_bytes();
            let ops_hash_header = block_reader.get_operations_hash()?;
            if ops_hash_stored != ops_hash_header {
                return Err(Error::Integrity(format!(
                    "Operations hash don't match: ops_hash_header={:?}, ops_hash_stored={:?}",
                    ops_hash_header, ops_hash_stored
                )));
            }
        }

        Ok(())
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

    pub fn new_genesis(cell: &FullCell) -> Result<BlockOwned, Error> {
        let operations = BlockOperations::empty();
        let block = Self::new_with_prev_info(cell, 0, 0, 0, &[], 0, operations)?;

        // TODO: Add master signature after doing https://github.com/appaquet/exocore/issues/46

        Ok(block)
    }

    pub fn new_with_prev_block<B>(
        cell: &Cell,
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
            cell,
            offset,
            depth,
            previous_offset,
            previous_hash,
            proposed_operation_id,
            operations,
        )
    }

    pub fn new_with_prev_info(
        cell: &Cell,
        offset: BlockOffset,
        depth: BlockDepth,
        previous_offset: BlockOffset,
        previous_hash: &[u8],
        proposed_operation_id: u64,
        operations: BlockOperations,
    ) -> Result<BlockOwned, Error> {
        let local_node = cell.local_node();
        let operations_data_size = operations.data.len() as u32;

        // initialize block
        let mut block_frame_builder = FrameBuilder::<block::Owned>::new();
        let mut block_builder: block::Builder = block_frame_builder.get_builder_typed();
        block_builder.set_offset(offset);
        block_builder.set_depth(depth + 1);
        block_builder.set_previous_offset(previous_offset);
        block_builder.set_previous_hash(previous_hash);
        block_builder.set_proposed_operation_id(proposed_operation_id);
        block_builder.set_proposed_node_id(local_node.id().to_str());
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
        let signature_frame = BlockSignatures::empty_signatures_for_nodes(cell)
            .to_frame_for_new_block(operations_data_size)?;

        // set required signatures size in block
        block_builder.set_signatures_size(signature_frame.frame_size() as BlockSignaturesSize);
        let block_frame = block_frame_builder.as_owned_framed(local_node.frame_signer())?;

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

    fn offset(&self) -> u64 {
        self.offset
    }

    fn block(&self) -> &Self::BlockType {
        &self.block
    }

    fn operations_data(&self) -> &[u8] {
        &self.operations_data
    }

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

    fn offset(&self) -> u64 {
        self.offset
    }

    fn block(&self) -> &Self::BlockType {
        &self.block
    }

    fn operations_data(&self) -> &[u8] {
        &self.operations_data
    }

    fn signatures(&self) -> &Self::SignaturesType {
        &self.signatures
    }
}

///
/// Block iterator over a slice of data.
///
pub struct ChainBlockIterator<'a> {
    current_offset: usize,
    data: &'a [u8],
    last_error: Option<Error>,
}

impl<'a> ChainBlockIterator<'a> {
    pub fn new(data: &'a [u8]) -> ChainBlockIterator<'a> {
        ChainBlockIterator {
            current_offset: 0,
            data,
            last_error: None,
        }
    }
}

impl<'a> Iterator for ChainBlockIterator<'a> {
    type Item = BlockRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_offset >= self.data.len() {
            return None;
        }

        let block_res = BlockRef::new(&self.data[self.current_offset..]);
        match block_res {
            Ok(block) => {
                self.current_offset += block.total_size() as usize;
                Some(block)
            }
            Err(Error::Framing(framed::Error::EOF(_))) => None,
            Err(other) => {
                self.last_error = Some(other);
                None
            }
        }
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
    pub fn empty() -> BlockOperations {
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
        let mut hasher = Sha3_256::new();
        let mut headers = Vec::new();
        let mut data = Vec::new();

        for operation in sorted_operations {
            let operation_reader = operation.get_typed_reader()?;
            let offset = data.len();
            let entry_data = operation.frame_data();
            hasher.input_signed_frame(&operation);
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
        let mut hasher = Sha3_256::new();
        for operation in sorted_operations {
            hasher.input_signed_frame(&operation);
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

    ///
    /// Create signatures with pre-allocated space for the number of nodes we have in
    /// the cell
    ///
    pub fn empty_signatures_for_nodes(cell: &Cell) -> BlockSignatures {
        let nodes = cell.nodes();
        let signatures = nodes
            .iter()
            .all()
            .map(|node| BlockSignature {
                node_id: node.id().clone(),
                signature: Signature::empty(),
            })
            .collect();

        BlockSignatures { signatures }
    }

    fn to_frame_builder(&self) -> FrameBuilder<block_signatures::Owned> {
        let mut frame_builder = FrameBuilder::new();

        let signatures_builder: block_signatures::Builder = frame_builder.get_builder_typed();
        let mut signatures_array = signatures_builder.init_signatures(self.signatures.len() as u32);
        for (i, signature) in self.signatures.iter().enumerate() {
            let mut signature_builder = signatures_array.reborrow().get(i as u32);
            signature.copy_into_builder(&mut signature_builder);
        }

        frame_builder
    }

    pub fn to_frame_for_new_block(
        &self,
        operations_size: BlockOperationsSize,
    ) -> Result<OwnedTypedFrame<block_signatures::Owned>, Error> {
        let mut signatures_frame_builder = self.to_frame_builder();
        let mut signatures_builder = signatures_frame_builder.get_builder_typed();
        signatures_builder.set_operations_size(operations_size);

        Ok(signatures_frame_builder.as_owned_unsigned_framed()?)
    }

    pub fn to_frame_for_existing_block(
        &self,
        block_reader: &block::Reader,
    ) -> Result<OwnedTypedFrame<block_signatures::Owned>, Error> {
        let expected_signatures_size = usize::from(block_reader.get_signatures_size());

        let mut signatures_frame_builder = self.to_frame_builder();
        let mut signatures_builder: block_signatures::Builder =
            signatures_frame_builder.get_builder_typed();
        signatures_builder.set_operations_size(block_reader.get_operations_size());
        let signatures_frame = signatures_frame_builder.as_owned_unsigned_framed()?;

        // make sure that the signatures frame size is not higher than pre-allocated space in block
        if signatures_frame.frame_size() > expected_signatures_size {
            return Err(Error::Integrity(format!(
                "Block local signatures are taking more space than allocated space ({} > {})",
                signatures_frame.frame_size(),
                block_reader.get_signatures_size()
            )));
        }

        // build a signatures frame that has the right amount of space as defined at the block
        let required_padding = (expected_signatures_size - signatures_frame.frame_size()) as u16;
        if required_padding > 0 {
            Ok(signatures_frame.padded(required_padding)?)
        } else {
            Ok(signatures_frame)
        }
    }
}

///
/// Represents a signature of the block by one node, using its own key to sign the block's hash.
///
pub struct BlockSignature {
    pub node_id: NodeId,
    pub signature: Signature,
}

impl BlockSignature {
    pub fn new(node_id: NodeId, signature: Signature) -> BlockSignature {
        BlockSignature { node_id, signature }
    }

    pub fn copy_into_builder(&self, builder: &mut block_signature::Builder) {
        builder.set_node_id(self.node_id.to_str());
        builder.set_node_signature(self.signature.get_bytes());
    }
}

///
/// Block related errors
///
#[derive(Clone, Debug, Fail)]
pub enum Error {
    #[fail(display = "Block integrity error: {}", _0)]
    Integrity(String),
    #[fail(display = "An offset is out of the block data: {}", _0)]
    OutOfBound(String),
    #[fail(display = "Error in message serialization")]
    Framing(#[fail(cause)] framed::Error),
    #[fail(display = "Error in capnp serialization: kind={:?} msg={}", _0, _1)]
    Serialization(capnp::ErrorKind, String),
    #[fail(display = "Field is not in capnp schema: code={}", _0)]
    SerializationNotInSchema(u16),
    #[fail(display = "Other operation error: {}", _0)]
    Other(String),
}

impl From<framed::Error> for Error {
    fn from(err: framed::Error) -> Self {
        Error::Framing(err)
    }
}

impl From<capnp::Error> for Error {
    fn from(err: capnp::Error) -> Self {
        Error::Serialization(err.kind, err.description)
    }
}

impl From<capnp::NotInSchema> for Error {
    fn from(err: capnp::NotInSchema) -> Self {
        Error::SerializationNotInSchema(err.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use exocore_common::node::LocalNode;

    #[test]
    fn should_allocate_signatures_space_for_nodes() -> Result<(), failure::Error> {
        let local_node = LocalNode::generate();
        let full_cell = FullCell::generate(local_node);
        let cell = full_cell.cell();
        let genesis_block = BlockOwned::new_genesis(&full_cell)?;

        let block_ops = BlockOperations::empty();
        let block1 = BlockOwned::new_with_prev_block(cell, &genesis_block, 0, block_ops)?;
        assert!(block1.signatures.frame_size() > 100);

        let node2 = LocalNode::generate();
        full_cell.nodes_mut().add(node2.node().clone());

        let block_ops = BlockOperations::empty();
        let block2 = BlockOwned::new_with_prev_block(cell, &genesis_block, 0, block_ops)?;
        assert!(block2.signatures.frame_size() > block1.signatures.frame_size());

        Ok(())
    }

    #[test]
    fn should_pad_signatures_from_block_signature_size() -> Result<(), failure::Error> {
        let local_node = LocalNode::generate();
        let full_cell = FullCell::generate(local_node);
        let cell = full_cell.cell();
        let genesis_block = BlockOwned::new_genesis(&full_cell)?;

        let block_ops = BlockOperations::empty();
        let block1 = BlockOwned::new_with_prev_block(cell, &genesis_block, 0, block_ops)?;
        let block1_reader: block::Reader = block1.block().get_typed_reader()?;

        // generate new signatures for existing block
        let block_signatures = BlockSignatures::new_from_signatures(Vec::new());
        let signatures_frame = block_signatures.to_frame_for_existing_block(&block1_reader)?;

        // new signatures frame should be the same size as the signatures specified in block
        assert_eq!(
            usize::from(block1_reader.get_signatures_size()),
            signatures_frame.frame_size()
        );

        Ok(())
    }
}
