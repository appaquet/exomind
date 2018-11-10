@0xf51296176d1e327e;

struct SegmentHeader {
  id @0: UInt64;
  firstOffset @1: UInt64;
  lastOffset @2: UInt64;
  frozen @3: Bool;
}

struct Block {
  offset @0: UInt64;
  hash @1: Text;
  entries @2: List(BlockEntry);
}

struct BlockEntry {
  hash @0: Text;
  data @1: Data;
}