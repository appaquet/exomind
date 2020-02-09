@0xc2b0b9df716f42ac;

using Chain = import "data_chain.capnp";
using Common = import "common.capnp";

#
# Pending
#
struct PendingSyncRequest {
    ranges             @0: List(PendingSyncRange);
    fromBlockHeight    @1: UInt64;
}

struct PendingSyncRange {
    fromOperation      @0: UInt64;
    fromIncluded       @1: Bool;
    toOperation        @2: UInt64;
    toIncluded         @3: Bool;

    operationsHash     @4: Data;
    operationsCount    @5: UInt32;

    operationsFrames   @6: List(Data); # Frames of Chain.ChainOperation
    operationsHeaders  @7: List(Chain.ChainOperationHeader);
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

    headers            @2: List(Chain.BlockPartialHeader);
    blocks             @3: List(Data); # BlockHeader + entries data + signatures
}
