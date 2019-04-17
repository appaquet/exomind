@0x8ee58f74f0999479;

struct Node {
    id          @0: Text;
    publicKey   @1: Text;

    address     @2: List(NodeAddress);
}

struct NodeAddress {
    data        @0: Text;
}