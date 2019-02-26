@0xf51296176d1e327e;

#
# Pending store
#
struct PendingOperation {
    pendingId              @0: UInt64;
    operationId            @1: UInt64;

    operation :union {
        entryNew           @2: OperationEntryNew;
        blockPropose       @3: OperationBlockPropose;
        blockSign          @4: OperationBlockSign;
        blockRefuse        @5: OperationBlockRefuse;
    }
}

struct OperationEntryNew {
    entryHeader     @0: EntryHeader;
    entryData       @1: Data;
}

struct OperationBlockPropose {
    offset           @0: UInt64;
    previousOffset   @1: UInt64;
    previousHash     @2: Data;
    entries          @3: List(EntryHeader);
}

struct OperationBlockSign {
    blockHeader    @0: BlockHeader;
    signatureData  @1: Data;
}

struct OperationBlockRefuse {
    blockHeader   @0: BlockHeader;
}


#
# Chain
#
struct Entry {
    id        @0: UInt64;
    time      @1: UInt64;
    sourceApp @2: Text;
    type      @3: EntryType;

    data      @4: Data;
}

struct EntryHeader {
    id        @0: UInt64;
    time      @1: UInt64;
    sourceApp @2: Text;
    type      @3: EntryType;
}

enum EntryType {
    cellData      @0;
    cellMeta      @1;
    entryCopy     @2;
    chainTruncate @3;
}

struct Block {
    offset         @0: UInt64;
    depth          @1: UInt64;
    previousOffset @2: UInt64;
    previousHash   @3: Data;
    signatureSize  @4: UInt16;

    sourceNodeId   @5: Text;

    entries        @6: List(Entry);
}

struct BlockHeader {
    offset         @0: UInt64;
    depth          @1: UInt64;
    previousOffset @2: UInt64;
    previousHash   @3: Data;
    signatureSize  @4: UInt16;

    sourceNodeId   @5: Text;
}

struct BlockSignatures {
    signatures @0: List(BlockSignature);
}

struct BlockSignature {
    nodeId         @0: Text;
    nodeSignature  @1: Data;
}
