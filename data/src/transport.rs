
use tokio::prelude::*;
use exocore_common::node::Node;

struct TransportContext {
    // TODO: Other nodes ? It's also in engine ...
}

// TODO: A-la-tokio Framed
pub trait Transport: Stream<Item = MessageType, Error = Error> + Sink<SinkItem = MessageType, SinkError = Error> {
    fn send_message(message: MessageType);
}

struct OutMessage {
    message_type: MessageType,
}

struct InMessage {
    from: Node,
    message_type: MessageType,
}

enum MessageType {
    PendingSyncEntries,
    ChainProposeBlock,
    ChainAcceptBlock,
    ChainGetSegmentsHeaders,
    ChainGetBlocksHeaders, // from, to
    ChainGetBlock,
}

enum Error  {

}


#[cfg(test)]
mod test {
    #[test]
    fn test_transport() {}
}
