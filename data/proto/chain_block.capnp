@0xf51296176d1e327e;

struct Block {
  offset @0: UInt64;
  hash @1: Text;
  entries @2: List(BlockEntry);
  signatureSize @3: UInt16;
}

struct BlockEntry {
  hash @0: Text;
  data @1: Data;
}

struct BlockSignatures {
  signatures @0: List(BlockSignature);
}

struct BlockSignature {
  nodeId @0: Text;
  offset @1: Text;
}
