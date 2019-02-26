@0xc2b0b9df716f42ac;

using Chain = import "data_chain.capnp";
using Common = import "common.capnp";

struct Envelope {
    layer  @0: UInt8;
    type   @1: UInt8;

    from   @2: Common.Node;
    to     @3: Common.Node;

    data   @4: Data;
}

struct EngineMessage {
    message :union {
        pendingSyncRequest    @0: PendingSyncRequest;
        pendingSyncResponse   @1: PendingSyncResponse;
        chainSyncRequest      @2: ChainSyncRequest;
    }
}

#
# Pending
#
struct PendingSyncRequest {
    ranges @0: List(PendingSyncRange);
}

struct PendingSyncResponse {
    ranges @0: List(PendingSyncRange);
}

struct PendingSyncRange {
    fromTime         @0: UInt64;
    toTime           @1: UInt64;

    requestedDetails @2: RequestedDetails;

    hash             @3: Data;
    operations       @4: List(Chain.PendingOperation);

    enum RequestedDetails {
      hash @0;
      headers @1;
      full @2;
    }
}

#
# Chain
#
struct ChainSyncRequest {
    fromOffset       @0: UInt64;
    toOffset         @1: UInt64;

    count            @2: UInt32;
    sampling         @3: UInt32;

    requestedDetails @4: RequestedDetails;

    headers          @5: List(Chain.BlockHeader);
    blocks           @6: List(Chain.Block);

    enum RequestedDetails {
      headers  @0;
      blocks   @1;
    }
}
