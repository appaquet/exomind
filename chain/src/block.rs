use std::borrow::Borrow;

use bytes::{Bytes, BytesMut};
use exocore_core::{
    cell::{Cell, CellNodeRole, FullCell, NodeId},
    framing::{
        CapnpFrameBuilder, FrameBuilder, FrameReader, MultihashFrame, MultihashFrameBuilder,
        PaddedFrame, PaddedFrameBuilder, SizedFrame, SizedFrameBuilder, TypedCapnpFrame,
    },
    sec::{
        hash::{Multihash, MultihashDigestExt, Sha3_256},
        signature::Signature,
    },
};
use exocore_protos::{
    capnp,
    generated::data_chain_capnp::{
        block_header, block_operation_header, block_signature, block_signatures,
    },
};

use crate::{data::Data, operation::OperationId};

pub type BlockOffset = u64;
pub type BlockHeight = u64;
pub type BlockOperationsSize = u32;
pub type BlockSignaturesSize = u16;

pub type BlockHeaderFrame<I> =
    TypedCapnpFrame<MultihashFrame<32, Sha3_256, SizedFrame<I>>, block_header::Owned>;
pub type BlockHeaderFrameBuilder =
    SizedFrameBuilder<MultihashFrameBuilder<32, Sha3_256, CapnpFrameBuilder<block_header::Owned>>>;
pub type SignaturesFrame<I> = TypedCapnpFrame<PaddedFrame<SizedFrame<I>>, block_signatures::Owned>;

/// A trait representing a block stored or to be stored in the chain.
/// It can either be a referenced block (`BlockRef`) or a in-memory block
/// (`BlockOwned`).
///
/// A block consists of 3 parts:
///  * Block header
///  * Operations' bytes (capnp serialized `chain_operation` frames)
///  * Block signatures
///
/// The block header and operations' data are the same on all nodes. Since a
/// node writes a block as soon as it has enough signatures, signatures can
/// differ from one node to the other. Signatures frame is pre-allocated, which
/// means that not all signatures may fit. But in theory, it should always
/// contain enough space for all nodes to add their own signature.
pub trait Block {
    type UnderlyingFrame: FrameReader<OwnedType = Bytes>;

    fn offset(&self) -> BlockOffset;
    fn header(&self) -> &BlockHeaderFrame<Self::UnderlyingFrame>;
    fn operations_data(&self) -> &[u8];
    fn signatures(&self) -> &SignaturesFrame<Self::UnderlyingFrame>;

    #[inline]
    fn total_size(&self) -> usize {
        self.header().whole_data_size()
            + self.operations_data().len()
            + self.signatures().whole_data_size()
    }

    #[inline]
    fn next_offset(&self) -> BlockOffset {
        self.offset() + self.total_size() as BlockOffset
    }

    #[inline]
    fn copy_data_into(&self, data: &mut [u8]) {
        let operations_data = self.operations_data();
        let operations_offset = self.header().whole_data_size();
        let signatures_offset = operations_offset + operations_data.len();

        self.header()
            .copy_into(data)
            .expect("Couldn't write block into given buffer");

        data[operations_offset..signatures_offset].copy_from_slice(operations_data);

        self.signatures()
            .copy_into(&mut data[signatures_offset..])
            .expect("Couldn't write signatures into given buffer");
    }

    fn as_data_vec(&self) -> Bytes {
        Bytes::from(
            [
                self.header().whole_data(),
                self.operations_data(),
                self.signatures().whole_data(),
            ]
            .concat(),
        )
    }

    fn to_owned(&self) -> DataBlock<Bytes> {
        DataBlock {
            offset: self.offset(),
            header: self.header().to_owned(),
            operations_data: Bytes::from(self.operations_data().to_vec()),
            signatures: self.signatures().to_owned(),
        }
    }

    fn get_height(&self) -> Result<BlockHeight, Error> {
        let reader = self.header().get_reader()?;
        Ok(reader.get_height())
    }

    fn get_proposed_operation_id(&self) -> Result<OperationId, Error> {
        let reader = self.header().get_reader()?;
        Ok(reader.get_proposed_operation_id())
    }

    fn operations_iter(&self) -> Result<BlockOperationsIterator, Error> {
        let block_header = self.header().get_reader()?;
        let operations_header = block_header
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
    ) -> Result<Option<crate::operation::OperationFrame<&[u8]>>, Error> {
        let block_header = self.header().get_reader()?;
        let operations_header: Vec<BlockOperationHeader> = block_header
            .get_operations_header()?
            .iter()
            .map(|reader| BlockOperationHeader::from_reader(&reader))
            .collect();

        let operation_index =
            operations_header.binary_search_by_key(&operation_id, |header| header.operation_id);

        if let Ok(operation_index) = operation_index {
            if operation_index > operations_header.len() {
                return Err(Error::OutOfBound(format!(
                    "Operation id={} of block={} had an invalid index {} out of {} operations",
                    operation_id,
                    self.offset(),
                    operation_index,
                    operations_header.len()
                )));
            }

            let frame = operations_header[operation_index].read_frame(self.operations_data())?;

            Ok(Some(frame))
        } else {
            Ok(None)
        }
    }

    fn validate<PB: Block>(&self, previous_block: Option<PB>) -> Result<(), Error> {
        // TODO: Signature ticket: https://github.com/appaquet/exocore/issues/46
        //       Should actually check signatures too

        let header = self.header();
        let header_reader: block_header::Reader = header.get_reader()?;

        header.inner().inner().verify()?;

        if let Some(previous_block) = previous_block {
            let previous_hash = previous_block.header().inner().inner().multihash_bytes();
            if previous_hash != header_reader.get_previous_hash()? {
                return Err(Error::Integrity(
                    "Hash of previous block doesn't match current block hash".to_string(),
                ));
            }
        }

        let sig_size_header = header_reader.get_signatures_size() as usize;
        let sig_size_stored = self.signatures().whole_data_size();
        if sig_size_header != sig_size_stored {
            return Err(Error::Integrity(format!(
                "Signatures size don't match: sig_size_header={}, sig_size_stored={}",
                sig_size_header, sig_size_stored
            )));
        }

        let ops_size_header = header_reader.get_operations_size() as usize;
        let ops_size_stored = self.operations_data().len();
        if ops_size_header != ops_size_stored {
            return Err(Error::Integrity(format!(
                "Operations size don't match: ops_size_header={}, ops_size_stored={}",
                ops_size_header, ops_size_stored
            )));
        }

        if ops_size_header > 0 {
            let operations = self.operations_iter()?;
            let ops_hash_stored = BlockOperations::hash_operations(operations)?;
            let ops_hash_header = Multihash::<32>::from_bytes(header_reader.get_operations_hash()?)
                .map_err(|err| {
                    Error::Integrity(format!("Hash in block header couldn't be decoded: {}", err))
                })?;

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

/// Reads block header frame from an underlying frame (or just data)
pub fn read_header_frame<I: FrameReader>(inner: I) -> Result<BlockHeaderFrame<I>, Error> {
    let sized_frame = SizedFrame::new(inner)?;
    let multihash_frame = MultihashFrame::new(sized_frame)?;
    let frame = TypedCapnpFrame::new(multihash_frame)?;
    Ok(frame)
}

pub fn read_header_frame_from_next_offset<I: FrameReader>(
    inner: I,
    next_offset: usize,
) -> Result<BlockHeaderFrame<I>, Error> {
    let sized_frame = SizedFrame::new_from_next_offset(inner, next_offset)?;
    let multihash_frame = MultihashFrame::new(sized_frame)?;
    let frame = TypedCapnpFrame::new(multihash_frame)?;
    Ok(frame)
}

pub fn build_header_frame(
    header: CapnpFrameBuilder<block_header::Owned>,
) -> BlockHeaderFrameBuilder {
    SizedFrameBuilder::new(MultihashFrameBuilder::<32, Sha3_256, _>::new(header))
}

/// Block from an arbitrary type of data.
pub struct DataBlock<D: Data> {
    pub offset: BlockOffset,
    pub header: BlockHeaderFrame<D>,
    pub operations_data: D,
    pub signatures: SignaturesFrame<D>,
}

impl<D: Data> DataBlock<D> {
    pub fn new(data: D) -> Result<DataBlock<D>, Error> {
        let header = read_header_frame(data.clone())?;
        let header_reader: block_header::Reader = header.get_reader()?;

        let operations_offset = header.whole_data_size();
        let operations_size = header_reader.get_operations_size() as usize;
        let signatures_offset = operations_offset + operations_size;
        let signatures_size = header_reader.get_signatures_size() as usize;

        if signatures_offset >= data.len() {
            return Err(Error::OutOfBound(format!(
                "Signature offset {} is after data len {}",
                signatures_offset,
                data.len()
            )));
        }

        let signatures_data = data.view(signatures_offset..signatures_offset + signatures_size);
        let signatures = BlockSignatures::read_frame(signatures_data)?;

        let operations_data = data.view(operations_offset..signatures_offset);

        Ok(DataBlock {
            offset: header_reader.get_offset(),
            header,
            operations_data,
            signatures,
        })
    }

    pub fn new_from_next_offset(data: D, next_offset: usize) -> Result<DataBlock<D>, Error> {
        let signatures = BlockSignatures::read_frame_from_next_offset(data.clone(), next_offset)?;
        let signatures_reader: block_signatures::Reader = signatures.get_reader()?;
        let signatures_offset = next_offset - signatures.whole_data_size();

        let operations_size = signatures_reader.get_operations_size() as usize;
        if operations_size > signatures_offset {
            return Err(Error::OutOfBound(format!(
                "Tried to read block from next offset {}, but its operations size would exceed beginning of file (operations_size={} signatures_offset={})",
                next_offset, operations_size, signatures_offset,
            )));
        }

        let operations_offset = signatures_offset - operations_size;
        let operations_data = data.view(operations_offset..signatures_offset);

        let header = read_header_frame_from_next_offset(data, operations_offset)?;
        let header_reader: block_header::Reader = header.get_reader()?;

        Ok(DataBlock {
            offset: header_reader.get_offset(),
            operations_data,
            header,
            signatures,
        })
    }
}

impl<D: Data> Block for DataBlock<D> {
    type UnderlyingFrame = D;

    fn offset(&self) -> u64 {
        self.offset
    }

    fn header(&self) -> &BlockHeaderFrame<Self::UnderlyingFrame> {
        &self.header
    }

    fn operations_data(&self) -> &[u8] {
        self.operations_data.slice(..)
    }

    fn signatures(&self) -> &SignaturesFrame<Self::UnderlyingFrame> {
        &self.signatures
    }
}

/// In-memory block.
pub struct BlockBuilder;

impl BlockBuilder {
    pub fn build(
        offset: BlockOffset,
        header: BlockHeaderFrame<Bytes>,
        operations_data: Bytes,
        signatures: SignaturesFrame<Bytes>,
    ) -> DataBlock<Bytes> {
        DataBlock {
            offset,
            header,
            operations_data,
            signatures,
        }
    }

    pub fn build_genesis(full_cell: &FullCell) -> Result<DataBlock<Bytes>, Error> {
        let operations = BlockOperations::empty();
        let block = Self::build_with_prev_info(full_cell.cell(), 0, 0, 0, &[], 0, operations)?;
        // TODO: Add master signature after doing https://github.com/appaquet/exocore/issues/46
        Ok(block)
    }

    pub fn build_with_prev_block<B>(
        cell: &Cell,
        previous_block: &B,
        proposed_operation_id: u64,
        operations: BlockOperations,
    ) -> Result<DataBlock<Bytes>, Error>
    where
        B: Block,
    {
        let previous_block_header_reader = previous_block.header().get_reader()?;

        let previous_offset = previous_block.offset();
        let previous_hash = previous_block.header().inner().inner().multihash_bytes();

        let offset = previous_block.next_offset();
        let height = previous_block_header_reader.get_height();

        Self::build_with_prev_info(
            cell,
            offset,
            height,
            previous_offset,
            previous_hash,
            proposed_operation_id,
            operations,
        )
    }

    pub fn build_with_prev_info(
        cell: &Cell,
        offset: BlockOffset,
        height: BlockHeight,
        previous_offset: BlockOffset,
        previous_hash: &[u8],
        proposed_operation_id: u64,
        operations: BlockOperations,
    ) -> Result<DataBlock<Bytes>, Error> {
        let local_node = cell.local_node();
        let operations_data_size = operations.data.len() as u32;

        // initialize block header
        let mut header_frame_builder = CapnpFrameBuilder::<block_header::Owned>::new();
        let mut header_msg_builder = header_frame_builder.get_builder();
        header_msg_builder.set_offset(offset);
        header_msg_builder.set_height(height + 1);
        header_msg_builder.set_previous_offset(previous_offset);
        header_msg_builder.set_previous_hash(previous_hash);
        header_msg_builder.set_proposed_operation_id(proposed_operation_id);
        header_msg_builder.set_proposed_node_id(&local_node.id().to_string());
        header_msg_builder.set_operations_size(operations_data_size);
        header_msg_builder.set_operations_hash(&operations.hash.to_bytes());

        let mut operations_builder = header_msg_builder
            .reborrow()
            .init_operations_header(operations.headers.len() as u32);
        for (i, header_builder) in operations.headers.iter().enumerate() {
            let mut entry_builder = operations_builder.reborrow().get(i as u32);
            header_builder.copy_into_builder(&mut entry_builder);
        }

        // create an empty signature for each node as a placeholder to find the size
        // required for signatures
        let signature_frame = BlockSignatures::empty_signatures_for_nodes(cell)
            .to_frame_for_new_block(operations_data_size)?;

        // set required signatures size in block
        header_msg_builder
            .set_signatures_size(signature_frame.whole_data_size() as BlockSignaturesSize);

        // serialize block header and then re-read it
        let final_frame_builder = build_header_frame(header_frame_builder);
        let final_frame_data = final_frame_builder.as_bytes();
        let block_header = read_header_frame(final_frame_data)?;

        Ok(DataBlock {
            offset,
            header: block_header,
            operations_data: operations.data,
            signatures: signature_frame,
        })
    }
}

/// Iterator over operations stored in a block.
pub struct BlockOperationsIterator<'a> {
    index: usize,
    operations_header: Vec<BlockOperationHeader>,
    operations_data: &'a [u8],
    last_error: Option<Error>,
}

impl<'a> Iterator for BlockOperationsIterator<'a> {
    type Item = crate::operation::OperationFrame<&'a [u8]>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.operations_header.len() {
            return None;
        }

        let header = &self.operations_header[self.index];
        self.index += 1;

        let frame_res = header.read_frame(self.operations_data);
        match frame_res {
            Ok(frame) => Some(frame),
            Err(err) => {
                self.last_error = Some(err);
                None
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.index, Some(self.operations_data.len()))
    }
}

/// Wraps operations header stored in a block.
pub struct BlockOperations {
    hash: Multihash<32>,
    headers: Vec<BlockOperationHeader>,
    data: Bytes,
}

impl BlockOperations {
    pub fn empty() -> BlockOperations {
        BlockOperations {
            hash: Multihash::default(),
            headers: Vec::new(),
            data: Bytes::new(),
        }
    }

    pub fn from_operations<I, M, F>(sorted_operations: I) -> Result<BlockOperations, Error>
    where
        I: Iterator<Item = M>,
        M: Borrow<crate::operation::OperationFrame<F>>,
        F: FrameReader,
    {
        let mut hasher = Sha3_256::default();
        let mut headers = Vec::new();
        let mut data = BytesMut::new();
        let mut last_operation_id = 0;

        for operation in sorted_operations {
            let operation = operation.borrow();
            let operation_reader = operation.get_reader()?;
            let offset = data.len();
            let entry_data = operation.whole_data();
            hasher.input_signed_frame(operation.inner().inner());
            data.extend_from_slice(entry_data);

            let operation_id = operation_reader.get_operation_id();
            if operation_id < last_operation_id {
                panic!(
                    "Tried to build a block from unsorted operations op={} < last={}",
                    operation_id, last_operation_id
                );
            }
            last_operation_id = operation_id;

            headers.push(BlockOperationHeader {
                operation_id,
                data_offset: offset as u32,
                data_size: (data.len() - offset) as u32,
            });
        }

        Ok(BlockOperations {
            hash: hasher.to_multihash(),
            headers,
            data: data.into(),
        })
    }

    pub fn hash_operations<I, M, F>(sorted_operations: I) -> Result<Multihash<32>, Error>
    where
        I: Iterator<Item = M>,
        M: Borrow<crate::operation::OperationFrame<F>>,
        F: FrameReader,
    {
        let mut hasher = Sha3_256::default();
        for operation in sorted_operations {
            hasher.input_signed_frame(operation.borrow().inner().inner());
        }
        Ok(hasher.to_multihash())
    }

    pub fn operations_count(&self) -> usize {
        self.headers.len()
    }

    pub fn operations_id(&self) -> impl Iterator<Item = OperationId> + '_ {
        self.headers.iter().map(|header| header.operation_id)
    }

    pub fn multihash(&self) -> Multihash<32> {
        self.hash
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }
}

/// Header of an operation stored within a block. It represents the position in
/// the bytes of the block.
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

    fn read_frame<'a>(
        &self,
        operations_data: &'a [u8],
    ) -> Result<crate::operation::OperationFrame<&'a [u8]>, Error> {
        let offset_from = self.data_offset as usize;
        let offset_to = self.data_offset as usize + self.data_size as usize;

        let frame =
            crate::operation::read_operation_frame(&operations_data[offset_from..offset_to])?;

        Ok(frame)
    }
}

/// Represents signatures stored in a block. Since a node writes a block as soon
/// as it has enough signatures, signatures can differ from one node to the
/// other. Signatures frame is pre-allocated, which means that not all
/// signatures may fit. But in theory, it should always contain enough space for
/// all nodes to add their own signature.
pub struct BlockSignatures {
    signatures: Vec<BlockSignature>,
}

impl BlockSignatures {
    pub fn new_from_signatures(signatures: Vec<BlockSignature>) -> BlockSignatures {
        BlockSignatures { signatures }
    }

    /// Create signatures with pre-allocated space for the number of nodes we
    /// have in the cell
    pub fn empty_signatures_for_nodes(cell: &Cell) -> BlockSignatures {
        let nodes = cell.nodes();
        let signatures = nodes
            .iter()
            .all()
            .filter(|cn| cn.has_role(CellNodeRole::Chain))
            .map(|cell_node| BlockSignature {
                node_id: cell_node.node().id().clone(),
                signature: Signature::empty(),
            })
            .collect();

        BlockSignatures { signatures }
    }

    fn to_frame_builder(&self) -> CapnpFrameBuilder<block_signatures::Owned> {
        let mut frame_builder = CapnpFrameBuilder::new();

        let signatures_builder: block_signatures::Builder = frame_builder.get_builder();
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
    ) -> Result<SignaturesFrame<Bytes>, Error> {
        let mut signatures_frame_builder = self.to_frame_builder();
        let mut signatures_builder = signatures_frame_builder.get_builder();
        signatures_builder.set_operations_size(operations_size);

        let frame_builder =
            SizedFrameBuilder::new(PaddedFrameBuilder::new(signatures_frame_builder, 0));
        let frame_data = frame_builder.as_bytes();
        Self::read_frame(frame_data)
    }

    pub fn to_frame_for_existing_block(
        &self,
        header_reader: &block_header::Reader,
    ) -> Result<SignaturesFrame<Bytes>, Error> {
        let expected_signatures_size = usize::from(header_reader.get_signatures_size());

        // create capnp frame
        let mut signatures_frame_builder = self.to_frame_builder();
        let mut signatures_builder = signatures_frame_builder.get_builder();
        signatures_builder.set_operations_size(header_reader.get_operations_size());
        let signatures_frame_data = signatures_frame_builder.as_bytes();
        let signatures_frame_data_len = signatures_frame_data.len();

        // create the enclosure frame (sized & padded)
        let mut frame_builder =
            SizedFrameBuilder::new(PaddedFrameBuilder::new(signatures_frame_data, 0));
        let frame_expected_size = frame_builder
            .expected_size()
            .expect("Frame should had been sized");

        // check if we need to add padding to match original signatures size
        if frame_expected_size < expected_signatures_size {
            let diff = expected_signatures_size - frame_expected_size;
            frame_builder
                .inner_mut()
                .set_minimum_size(signatures_frame_data_len + diff);
        }

        // we build the frame and re-read it
        let signatures_frame = Self::read_frame(frame_builder.as_bytes())?;

        // make sure that the signatures frame size is the same as the one in block
        // header
        if signatures_frame.whole_data_size() != expected_signatures_size {
            return Err(Error::Integrity(format!(
                "Block local signatures isn't the same size as expected (this={} expected={})",
                signatures_frame.whole_data_size(),
                header_reader.get_signatures_size()
            )));
        }

        Ok(signatures_frame)
    }

    pub fn read_frame<I: FrameReader>(inner: I) -> Result<SignaturesFrame<I>, Error> {
        let sized_frame = SizedFrame::new(inner)?;
        let padded_frame = PaddedFrame::new(sized_frame)?;
        let frame = TypedCapnpFrame::new(padded_frame)?;
        Ok(frame)
    }

    pub fn read_frame_from_next_offset<I: FrameReader>(
        inner: I,
        next_offset: usize,
    ) -> Result<SignaturesFrame<I>, Error> {
        let sized_frame = SizedFrame::new_from_next_offset(inner, next_offset)?;
        let padded_frame = PaddedFrame::new(sized_frame)?;
        let frame = TypedCapnpFrame::new(padded_frame)?;
        Ok(frame)
    }
}

/// Represents a signature of the block by one node, using its own key to sign
/// the block's hash.
pub struct BlockSignature {
    pub node_id: NodeId,
    pub signature: Signature,
}

impl BlockSignature {
    pub fn new(node_id: NodeId, signature: Signature) -> BlockSignature {
        BlockSignature { node_id, signature }
    }

    pub fn copy_into_builder(&self, builder: &mut block_signature::Builder) {
        builder.set_node_id(&self.node_id.to_string());
        builder.set_node_signature(self.signature.get_bytes());
    }
}

/// Block related errors
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Block integrity error: {0}")]
    Integrity(String),

    #[error("An offset is out of the block data: {0}")]
    OutOfBound(String),

    #[error("Operations related error: {0}")]
    Operation(#[from] crate::operation::Error),

    #[error("Framing error: {0}")]
    Framing(#[from] exocore_core::framing::Error),

    #[error("Error in capnp serialization: {0}")]
    Serialization(#[from] capnp::Error),

    #[error("Field is not in capnp schema: code={0}")]
    SerializationNotInSchema(u16),

    #[error("Other operation error: {0}")]
    Other(String),
}

impl From<capnp::NotInSchema> for Error {
    fn from(err: capnp::NotInSchema) -> Self {
        Error::SerializationNotInSchema(err.0)
    }
}

#[cfg(test)]
mod tests {
    use exocore_core::{
        cell::{FullCell, LocalNode, Node},
        framing::FrameReader,
    };

    use super::*;
    use crate::{
        block::{Block, BlockBuilder, BlockOperations},
        data::RefData,
        operation::OperationBuilder,
    };

    #[test]
    fn block_create_and_read() -> anyhow::Result<()> {
        let local_node = LocalNode::generate();
        let full_cell = FullCell::generate(local_node.clone())?;

        {
            // local node is chain node
            let mut nodes = full_cell.cell().nodes_mut();
            let local_cell_node = nodes.get_mut(local_node.id()).unwrap();
            local_cell_node.add_role(CellNodeRole::Chain);

            // second node is not chain node
            nodes.add(Node::generate_temporary());
        }

        let genesis = BlockBuilder::build_genesis(&full_cell)?;

        let operations = vec![
            OperationBuilder::new_entry(123, local_node.id(), b"some_data")
                .sign_and_build(&local_node)?
                .frame,
        ];
        let operations = BlockOperations::from_operations(operations.into_iter())?;

        let second_block =
            BlockBuilder::build_with_prev_block(full_cell.cell(), &genesis, 0, operations)?;

        let mut data = [0u8; 5000];
        second_block.copy_data_into(&mut data);

        let block_data = RefData::new(&data[0..second_block.total_size()]);
        let read_second_block = DataBlock::new(block_data)?;
        assert_eq!(
            second_block.header.whole_data(),
            read_second_block.header.whole_data()
        );

        assert_eq!(
            second_block.operations_data.as_ref(),
            read_second_block.operations_data.slice(..),
        );
        assert_eq!(
            second_block.signatures.whole_data(),
            read_second_block.signatures.whole_data()
        );

        let header_reader = second_block.header.get_reader()?;
        assert_eq!(header_reader.get_offset(), genesis.next_offset());
        assert_eq!(
            header_reader.get_signatures_size(),
            second_block.signatures.whole_data_size() as u16
        );
        assert_eq!(
            header_reader.get_operations_size(),
            second_block.operations_data.len() as u32
        );

        let signatures_reader = second_block.signatures.get_reader()?;
        assert_eq!(
            signatures_reader.get_operations_size(),
            second_block.operations_data.len() as u32
        );

        // 1 signature only since only our nodes is chain node
        let signatures = signatures_reader.get_signatures()?;
        assert_eq!(signatures.len(), 1);

        Ok(())
    }

    #[test]
    fn block_operations() -> anyhow::Result<()> {
        let local_node = LocalNode::generate();
        let full_cell = FullCell::generate(local_node.clone())?;
        let genesis = BlockBuilder::build_genesis(&full_cell)?;

        // 0 operations
        let block = BlockBuilder::build_with_prev_block(
            full_cell.cell(),
            &genesis,
            0,
            BlockOperations::empty(),
        )?;
        assert_eq!(block.operations_iter()?.count(), 0);

        // 5 operations
        let operations = (0..5).map(|i| {
            OperationBuilder::new_entry(i, local_node.id(), b"op1")
                .sign_and_build(&local_node)
                .unwrap()
                .frame
        });

        let block_operations = BlockOperations::from_operations(operations)?;
        let block =
            BlockBuilder::build_with_prev_block(full_cell.cell(), &genesis, 0, block_operations)?;
        assert_eq!(block.operations_iter()?.count(), 5);

        Ok(())
    }

    #[test]
    fn should_allocate_signatures_space_for_nodes() -> anyhow::Result<()> {
        let local_node = LocalNode::generate();
        let full_cell = FullCell::generate(local_node.clone())?;
        let cell = full_cell.cell();

        let node2 = {
            // local node is chain node
            let mut nodes = cell.nodes_mut();
            let local_cell_node = nodes.get_mut(local_node.id()).unwrap();
            local_cell_node.add_role(CellNodeRole::Chain);

            // second node is not chain node
            let node2 = Node::generate_temporary();
            nodes.add(node2.clone());
            node2
        };

        let genesis_block = BlockBuilder::build_genesis(&full_cell)?;

        // only first node is chain node
        let block_ops = BlockOperations::empty();
        let block1 = BlockBuilder::build_with_prev_block(cell, &genesis_block, 0, block_ops)?;
        assert!(block1.signatures.whole_data_size() > 100);

        // make second node chain node, should now have more signature size
        {
            let mut nodes = cell.nodes_mut();
            let cell_node_2 = nodes.get_mut(node2.id()).unwrap();
            cell_node_2.add_role(CellNodeRole::Chain);
        }

        let block_ops = BlockOperations::empty();
        let block2 = BlockBuilder::build_with_prev_block(cell, &genesis_block, 0, block_ops)?;
        assert!(block2.signatures.whole_data_size() > block1.signatures.whole_data_size());

        Ok(())
    }

    #[test]
    fn should_pad_signatures_from_block_signature_size() -> anyhow::Result<()> {
        let local_node = LocalNode::generate();
        let full_cell = FullCell::generate(local_node)?;
        let cell = full_cell.cell();
        let genesis_block = BlockBuilder::build_genesis(&full_cell)?;

        let block_ops = BlockOperations::empty();
        let block1 = BlockBuilder::build_with_prev_block(cell, &genesis_block, 0, block_ops)?;
        let block1_reader: block_header::Reader = block1.header().get_reader()?;

        // generate new signatures for existing block
        let block_signatures = BlockSignatures::new_from_signatures(Vec::new());
        let signatures_frame = block_signatures.to_frame_for_existing_block(&block1_reader)?;

        // new signatures frame should be the same size as the signatures specified in
        // block
        assert_eq!(
            usize::from(block1_reader.get_signatures_size()),
            signatures_frame.whole_data_size()
        );

        Ok(())
    }
}
