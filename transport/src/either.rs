use crate::error::Error;
use crate::transport::TransportHandleOnStart;
use crate::{InEvent, OutEvent, TransportHandle};
use exocore_common::node::NodeId;
use exocore_common::utils::completion_notifier::{CompletionListener, CompletionNotifier};
use exocore_common::utils::futures::OwnedSpawnSet;
use futures::channel::mpsc;
use futures::compat::{Compat01As03, Future01CompatExt};
use futures::prelude::*;
use futures::{Future, FutureExt, Sink, SinkExt, Stream, StreamExt};
use pin_project::pin_project;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::{Arc, RwLock, Weak};
use std::task::{Context, Poll};

/// Transport handle that wraps 2 other transport handles. When it receives events,
/// it notes from which transport it came so that replies can be sent back via the same
/// transport.
///
/// !! If we never received an event for a node, it will automatically select the first handle !!
#[pin_project]
pub struct EitherTransportHandle<TLeft, TRight>
where
    TLeft: TransportHandle,
    TRight: TransportHandle,
{
    #[pin]
    left: TLeft,
    #[pin]
    right: TRight,
    inner: Arc<RwLock<Inner>>,
    #[pin]
    completion_listener: Compat01As03<CompletionListener<(), Error>>,
    owned_spawn_set: OwnedSpawnSet<()>,
}

impl<TLeft, TRight> EitherTransportHandle<TLeft, TRight>
where
    TLeft: TransportHandle,
    TRight: TransportHandle,
{
    pub fn new(left: TLeft, right: TRight) -> EitherTransportHandle<TLeft, TRight> {
        let (completion_notifier, completion_listener) = CompletionNotifier::new_with_listener();

        let owned_spawn_set = OwnedSpawnSet::new();
        EitherTransportHandle {
            left,
            right,
            inner: Arc::new(RwLock::new(Inner {
                node_map: HashMap::new(),
                completion_notifier,
            })),
            completion_listener: completion_listener.compat(),
            owned_spawn_set,
        }
    }
}

impl<TLeft, TRight> TransportHandle for EitherTransportHandle<TLeft, TRight>
where
    TLeft: TransportHandle,
    TRight: TransportHandle,
{
    type Sink = Box<dyn Sink<OutEvent, Error = Error> + Send + Unpin + 'static>;
    type Stream = Box<dyn Stream<Item = InEvent> + Send + Unpin + 'static>;

    fn on_started(&self) -> TransportHandleOnStart {
        let left = self.left.on_started();
        let right = self.right.on_started();

        Box::new(future::select(left, right).map(|_| ()))
    }

    fn get_sink(&mut self) -> Self::Sink {
        // dispatch incoming events from left transport to handle
        let mut left = self.left.get_sink();
        let (left_sender, mut left_receiver) = mpsc::unbounded();
        let weak_inner2 = Arc::downgrade(&self.inner);
        self.owned_spawn_set.spawn(
            async move {
                while let Some(event) = left_receiver.next().await {
                    left.send(event).await?;
                }
                Ok(())
            }
            .map_err(move |err| {
                error!("Error in sending to either side: {}", err);
                let _ = Inner::maybe_complete_error(&weak_inner2, err);
            })
            .map(|_| ()),
        );

        // dispatch incoming events from right transport to handle
        let mut right = self.right.get_sink();
        let (right_sender, mut right_receiver) = mpsc::unbounded();
        let weak_inner2 = Arc::downgrade(&self.inner);
        self.owned_spawn_set.spawn(
            async move {
                while let Some(event) = right_receiver.next().await {
                    right.send(event).await?;
                }
                Ok(())
            }
            .map_err(move |err| {
                error!("Error in sending to either side: {}", err);
                let _ = Inner::maybe_complete_error(&weak_inner2, err);
            })
            .map(|_| ()),
        );

        // redirect outgoing events coming from handles to underlying transports
        let (sender, mut receiver) = mpsc::unbounded::<OutEvent>();
        let weak_inner1 = Arc::downgrade(&self.inner);
        let weak_inner2 = Arc::downgrade(&self.inner);
        self.owned_spawn_set.spawn(
            async move {
                while let Some(event) = receiver.next().await {
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
                }

                Ok::<_, Error>(())
            }
            .map_err(move |err| {
                error!("Error in sending to either side: {}", err);
                let _ = Inner::maybe_complete_error(&weak_inner2, err);
            })
            .map(|_| ()),
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

        Box::new(futures::stream::select(left, right))
    }
}

impl<TLeft, TRight> Future for EitherTransportHandle<TLeft, TRight>
where
    TLeft: TransportHandle,
    TRight: TransportHandle,
{
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        if let Poll::Ready(_) = this.left.poll(cx) {
            return Poll::Ready(());
        }

        if let Poll::Ready(_) = this.right.poll(cx) {
            return Poll::Ready(());
        }

        if let Poll::Ready(_) = this.completion_listener.poll(cx) {
            return Poll::Ready(());
        }

        Poll::Pending
    }
}

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
    use exocore_common::utils::futures::Runtime;
    use futures01::future;

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
            .block_on(future::lazy(|| {
                future::ok::<_, Error>(TestableTransportHandle::new(EitherTransportHandle::new(
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
