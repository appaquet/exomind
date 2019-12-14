use crate::error::Error;
use crate::{InEvent, OutEvent, TransportHandle};
use exocore_common::node::NodeId;
use exocore_common::utils::completion_notifier::{
    CompletionError, CompletionListener, CompletionNotifier,
};
use exocore_common::utils::futures::spawn_future_01;
use futures01::prelude::*;
use futures01::sync::mpsc;
use std::collections::HashMap;
use std::sync::{Arc, RwLock, Weak};

///
/// Transport handle that wraps 2 other transport handles. When it receives events,
/// it notes from which transport it came so that replies can be sent back via the same
/// transport.
///
/// !! If we never received an event for a node, it will automatically select the first handle !!
///
pub struct EitherTransportHandle<TLeft: TransportHandle, TRight: TransportHandle> {
    left: TLeft,
    right: TRight,
    inner: Arc<RwLock<Inner>>,
    completion_listener: CompletionListener<(), Error>,
}

impl<TLeft: TransportHandle, TRight: TransportHandle> EitherTransportHandle<TLeft, TRight> {
    pub fn new(left: TLeft, right: TRight) -> EitherTransportHandle<TLeft, TRight> {
        let (completion_notifier, completion_listener) = CompletionNotifier::new_with_listener();

        EitherTransportHandle {
            left,
            right,
            inner: Arc::new(RwLock::new(Inner {
                node_map: HashMap::new(),
                completion_notifier,
            })),
            completion_listener,
        }
    }
}

impl<TLeft: TransportHandle, TRight: TransportHandle> TransportHandle
    for EitherTransportHandle<TLeft, TRight>
{
    // TODO: Figure out static types https://github.com/appaquet/exocore/issues/125
    type StartFuture = Box<dyn Future<Item = (), Error = Error> + Send + 'static>;
    type Sink = Box<dyn Sink<SinkItem = OutEvent, SinkError = Error> + Send + 'static>;
    type Stream = Box<dyn Stream<Item = InEvent, Error = Error> + Send + 'static>;

    fn on_start(&self) -> Self::StartFuture {
        let left = self.left.on_start();
        let right = self.right.on_start();

        Box::new(
            left.select(right)
                .map(|(res, _)| res)
                .map_err(|(err, _)| err),
        )
    }

    fn get_sink(&mut self) -> Self::Sink {
        // dispatch incoming events from left transport to handle
        let left = self.left.get_sink();
        let (left_sender, left_receiver) = mpsc::unbounded();
        let weak_inner = Arc::downgrade(&self.inner);
        spawn_future_01(
            left_receiver
                .map_err(|_| Error::Other("Left side channel has been dropped".to_owned()))
                .forward(left)
                .map(|_| ())
                .map_err(move |err| {
                    error!("Got an error sending to left sink: {}", err);
                    let _ = Inner::maybe_complete_error(&weak_inner, err);
                }),
        );

        // dispatch incoming events from right transport to handle
        let right = self.right.get_sink();
        let (right_sender, right_receiver) = mpsc::unbounded();
        let weak_inner = Arc::downgrade(&self.inner);
        spawn_future_01(
            right_receiver
                .map_err(|_| Error::Other("Right side channel has been dropped".to_owned()))
                .forward(right)
                .map(|_| ())
                .map_err(move |err| {
                    error!("Got an error sending to right sink: {}", err);
                    let _ = Inner::maybe_complete_error(&weak_inner, err);
                }),
        );

        // redirect outgoing events coming from handles to underlying transports
        let (sender, receiver) = mpsc::unbounded::<OutEvent>();
        let weak_inner1 = Arc::downgrade(&self.inner);
        let weak_inner2 = Arc::downgrade(&self.inner);
        spawn_future_01(
            receiver
                .map_err(|_| Error::Other("Error in incoming channel stream".to_string()))
                .for_each(move |event| {
                    let side = match &event {
                        OutEvent::Message(msg) => {
                            let node = msg.to.first().ok_or_else(|| {
                                Error::Other(
                                    "Out event didn't have any destination node".to_string(),
                                )
                            })?;
                            Inner::get_side(&weak_inner1, node.id())?
                        }
                    };

                    // default to left side if we didn't find node
                    if let Some(Side::Right) = side {
                        right_sender.unbounded_send(event).map_err(|err| {
                            Error::Other(format!("Couldn't send to right sink: {}", err))
                        })?;
                    } else {
                        left_sender.unbounded_send(event).map_err(|err| {
                            Error::Other(format!("Couldn't send to left sink: {}", err))
                        })?;
                    }

                    Ok(())
                })
                .map_err(move |err| {
                    error!("Got an error sending to sink: {}", err);
                    let _ = Inner::maybe_complete_error(&weak_inner2, err);
                }),
        );

        Box::new(sender.sink_map_err(|err| Error::Other(format!("Error in either sink: {}", err))))
    }

    fn get_stream(&mut self) -> Self::Stream {
        let weak_inner = Arc::downgrade(&self.inner);
        let left = self.left.get_stream().map(move |event| {
            if let Err(err) = Inner::maybe_add_node(&weak_inner, &event, Side::Left) {
                error!("Error saving node's transport from left side: {}", err);
                let _ = Inner::maybe_complete_error(&weak_inner, err);
            }
            event
        });

        let weak_inner = Arc::downgrade(&self.inner);
        let right = self.right.get_stream().map(move |event| {
            if let Err(err) = Inner::maybe_add_node(&weak_inner, &event, Side::Right) {
                error!("Error saving node's transport from right side: {}", err);
                let _ = Inner::maybe_complete_error(&weak_inner, err);
            }
            event
        });

        Box::new(left.select(right))
    }
}

impl<TLeft: TransportHandle, TRight: TransportHandle> Future
    for EitherTransportHandle<TLeft, TRight>
{
    type Item = ();
    type Error = Error;

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        if let Async::Ready(_) = self.left.poll()? {
            return Ok(Async::Ready(()));
        }

        if let Async::Ready(_) = self.right.poll()? {
            return Ok(Async::Ready(()));
        }

        let completion_poll = self.completion_listener.poll().map_err(|err| match err {
            CompletionError::UserError(err) => err,
            CompletionError::Dropped => {
                Error::Other("Completion notifier has been dropped".to_owned())
            }
        })?;
        if let Async::Ready(_) = completion_poll {
            return Ok(Async::Ready(()));
        }

        Ok(Async::NotReady)
    }
}

///
/// Inner shared instance of the handle
///
struct Inner {
    node_map: HashMap<NodeId, Side>,
    completion_notifier: CompletionNotifier<(), Error>,
}

impl Inner {
    fn get_side(weak_inner: &Weak<RwLock<Inner>>, node_id: &NodeId) -> Result<Option<Side>, Error> {
        let inner = weak_inner.upgrade().ok_or(Error::Upgrade)?;
        let inner = inner.read()?;
        Ok(inner.node_map.get(node_id).cloned())
    }

    fn maybe_add_node(
        weak_inner: &Weak<RwLock<Inner>>,
        event: &InEvent,
        side: Side,
    ) -> Result<(), Error> {
        let node_id = match event {
            InEvent::Message(msg) => msg.from.id(),
            _ => return Ok(()),
        };

        let inner = weak_inner.upgrade().ok_or(Error::Upgrade)?;

        {
            // check if node is already in map with read lock
            let inner = inner.read()?;
            if inner.node_map.get(node_id) == Some(&side) {
                return Ok(());
            }
        }

        {
            // if we're here, node is not in the map and need the write lock
            let mut inner = inner.write()?;
            inner.node_map.insert(node_id.clone(), side);
        }

        Ok(())
    }

    fn maybe_complete_error(weak_inner: &Weak<RwLock<Inner>>, error: Error) -> Result<(), Error> {
        let inner = weak_inner.upgrade().ok_or(Error::Upgrade)?;
        let inner = inner.read()?;
        inner.completion_notifier.complete(Err(error));
        Ok(())
    }
}

#[derive(PartialEq, Clone, Copy)]
enum Side {
    Left,
    Right,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::{MockTransport, TestableTransportHandle};
    use crate::TransportLayer::Index;
    use exocore_common::node::LocalNode;
    use exocore_common::tests_utils::{expect_result, result_assert_false, result_assert_true};
    use futures01::future;
    use tokio::runtime::Runtime;

    #[test]
    fn test_send_and_receive() -> Result<(), failure::Error> {
        let mut rt = Runtime::new()?;

        let mock_transport1 = MockTransport::default();
        let mock_transport2 = MockTransport::default();
        let node1 = LocalNode::generate();
        let node2 = LocalNode::generate();

        // create 2 different kind of transports
        // on node 1, we use it combined using the EitherTransportHandle
        let node1_transport1 = mock_transport1.get_transport(node1.clone(), Index);
        let node1_transport2 = mock_transport2.get_transport(node1.clone(), Index);
        let mut node1_either = rt
            .block_on::<_, _, Error>(future::lazy(|| {
                future::ok(TestableTransportHandle::new(EitherTransportHandle::new(
                    node1_transport1,
                    node1_transport2,
                )))
            }))
            .unwrap();
        node1_either.start(&mut rt);

        // on node 2, we use transports independently
        let mut node2_transport1 = mock_transport1
            .get_transport(node2.clone(), Index)
            .into_testable();
        node2_transport1.start(&mut rt);
        let mut node2_transport2 = mock_transport2
            .get_transport(node2.clone(), Index)
            .into_testable();
        node2_transport2.start(&mut rt);

        // since node1 has never sent message, it will send to node 2 via transport 1 (left side)
        node1_either.send_test_message(&mut rt, node2.node(), 1);
        expect_result::<_, _, failure::Error>(|| {
            let transport1_has_message = node2_transport1.has_message()?;
            let transport2_has_message = node2_transport2.has_message()?;
            result_assert_true(transport1_has_message)?;
            result_assert_false(transport2_has_message)?;
            Ok(())
        });

        // sending to node1 via both transport should be received
        node2_transport1.send_test_message(&mut rt, node1.node(), 2);
        let (from, msg) = node1_either.receive_test_message(&mut rt);
        assert_eq!(&from, node2.id());
        assert_eq!(msg, 2);
        node2_transport2.send_test_message(&mut rt, node1.node(), 3);
        let (from, msg) = node1_either.receive_test_message(&mut rt);
        assert_eq!(&from, node2.id());
        assert_eq!(msg, 3);

        // sending to node2 should now be sent via transport 2 since its last used transport (right side)
        node1_either.send_test_message(&mut rt, node2.node(), 4);
        let (from, msg) = node2_transport2.receive_test_message(&mut rt);
        assert_eq!(&from, node1.id());
        assert_eq!(msg, 4);

        Ok(())
    }
}
