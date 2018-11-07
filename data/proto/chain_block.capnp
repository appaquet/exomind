@0x9eb32e19f86ee174;

struct Person {
  id @0 :UInt32;
  name @1 :Text;
  email @2 :Text;
  phones @3 :List(PhoneNumber);

  struct PhoneNumber {
    number @0 :Text;
    type @1 :Type;

    enum Type {
      mobile @0;
      home @1;
      work @2;
    }
  }

  employment :union {
    unemployed @4 :Void;
    employer @5 :Text;
    school @6 :Text;
    selfEmployed @7 :Void;
    # We assume that a person is only one of these.
  }
}

struct AddressBook {
  people @0 :List(Person);
}

struct Block {
  offset @0: UInt64;
  size @1: UInt32;
  hash @2: Text;
  entries @3: List(BlockEntry);
}

struct BlockEntry {
  hash @0: Text;
  data @1: Data;
}