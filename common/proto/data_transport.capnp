@0xc2b0b9df716f42ac;

using Chain = import "data_chain.capnp";
using Common = import "common.capnp";

struct Envelope {
    type @0: UInt8;
    from @1: Common.Node;
}

struct PendingSyncRequest {
    ranges @0: List(PendingSyncRange);
}

struct PendingSyncResponse {
    ranges @0: List(PendingSyncRange);
}

struct PendingSyncRange {
    fromTime @0: UInt64;
    toTime @1: UInt64;

    hashOnly @2: Bool;
    hash @3: Data;

    operations @4: List(Chain.PendingOperation);
}

