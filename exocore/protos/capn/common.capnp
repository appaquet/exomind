@0x8ee58f74f0999479;

#
# Message envelope for transport between nodes / service
#
struct Envelope {
    cellId             @0: Data;
    service            @1: UInt8;
    type               @2: UInt16;
    rendezVousId       @3: UInt64;
    fromNodeId         @4: Text;

    data               @5: Data; # Usually a frame
}


struct Node {
    id          @0: Text;
    publicKey   @1: Text;

    address     @2: List(NodeAddress);
}

struct NodeAddress {
    data        @0: Text;
}


