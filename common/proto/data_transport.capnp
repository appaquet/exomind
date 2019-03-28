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
        chainSyncRequest      @1: ChainSyncRequest;
    }
}

#
# Pending
#
struct PendingSyncRequest {
    ranges @0: List(PendingSyncRange);
}

struct PendingSyncRange {
    fromOperation      @0: UInt64;
    toOperation        @1: UInt64;

    operationsHash     @2: Data;
    operationsCount    @3: UInt32;

    operations         @4: List(Data); # Frames of Chain.PendingOperation
    operationsHeaders  @5: List(Chain.PendingOperationHeader);

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
    sample           @3: UInt32;

    requestedDetails @4: RequestedDetails;

    headers          @5: List(Chain.BlockHeader);
    blocks           @6: List(Chain.Block);

    enum RequestedDetails {
      headers  @0;
      blocks   @1;
    }
}
