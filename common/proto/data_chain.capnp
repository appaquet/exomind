@0xf51296176d1e327e;

struct PendingOperation {
    uid @0: UInt64;
    time @1: UInt64;

    operation :union {
        newEntry @2: OperationNewEntry;
        blockProposal @3: OperationBlockProposal;
        blockSignature @4: OperationBlockSignature;
    }
}

struct OperationNewEntry {
    entry @0: Entry;
}

struct OperationBlockProposal {
    entries @0: List(EntryHeader);
}

struct OperationBlockSignature {
    blockOffset @0: UInt64;
}



struct Entry {
    header @0: EntryHeader;
    data @1: Data;
}

struct EntryHeader {
    uid @0: UInt64;
    time @1: UInt64;
    sourceApp @2: Text;
    hash @3: Text;
}


struct Block {
    offset @0: UInt64;
    hash @1: Text;
    previousBlockOffset @2: UInt64;
    previousBlockHash @3: Text;

    entries @4: List(Entry);
    signatureSize @5: UInt16;
}

struct BlockSignatures {
    signatures @0: List(BlockSignature);
}

struct BlockSignature {
    nodeId @0: Text;
    offset @1: Text;
}
