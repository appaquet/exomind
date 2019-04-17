@0xc2b0b9df716f42ac;

using Chain = import "data_chain.capnp";
using Common = import "common.capnp";

struct Envelope {
    layer              @0: UInt8;
    type               @1: UInt16;
    fromNode           @2: Text;

    data               @3: Data;
}

#
# Pending
#
struct PendingSyncRequest {
    ranges             @0: List(PendingSyncRange);
}

struct PendingSyncRange {
    fromOperation      @0: UInt64;
    fromIncluded       @1: Bool;
    toOperation        @2: UInt64;
    toIncluded         @3: Bool;

    operationsHash     @4: Data;
    operationsCount    @5: UInt32;

    operations         @6: List(Data); # Frames of Chain.PendingOperation
    operationsHeaders  @7: List(Chain.PendingOperationHeader);
}

#
# Chain
#
struct ChainSyncRequest {
    fromOffset         @0: UInt64;
    toOffset           @1: UInt64;

    requestedDetails   @2: RequestedDetails;

    enum RequestedDetails {
      headers          @0;
      blocks           @1;
    }
}

struct ChainSyncResponse {
    fromOffset         @0: UInt64;
    toOffset           @1: UInt64;

    headers            @2: List(Chain.BlockHeader);
    blocks             @3: List(Data); # Block + entries data + signatures
}
