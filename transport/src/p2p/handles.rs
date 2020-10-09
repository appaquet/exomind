use std::collections::HashMap;
use std::pin::Pin;
use std::sync::{RwLock, Weak};
use std::task::{Context, Poll};

use futures::channel::mpsc;
use futures::channel::mpsc::SendError;
use futures::prelude::*;
use futures::sink::SinkMapErr;
use futures::{FutureExt, SinkExt};

use exocore_core::cell::{Cell, CellId};
use exocore_core::cell::{Node, NodeId};
use exocore_core::utils::handle_set::Handle;

use crate::transport::{InEvent, OutEvent, TransportHandleOnStart};
use crate::Error;
use crate::{ServiceType, TransportServiceHandle};

/// Transport handles created on the `Libp2pTransport` to be used by services.
///
/// A transport can be used by multiple services from multiple cells, so
/// multiple handles for the same service, but on different cells may be
/// created.
#[derive(Default)]
pub(super) struct ServiceHandles {
    pub(super) service_handles: HashMap<(CellId, ServiceType), ServiceHandle>,
}

impl ServiceHandles {
    pub(super) fn push_handle(
        &mut self,
        cell: Cell,
        service_type: ServiceType,
        in_sender: mpsc::Sender<InEvent>,
        out_receiver: mpsc::Receiver<OutEvent>,
    ) {
        let handle = ServiceHandle {
            cell: cell.clone(),
            in_sender,
            out_receiver: Some(out_receiver),
        };

        let key = (cell.id().clone(), service_type);
        self.service_handles.insert(key, handle);
    }

    pub(super) fn all_peer_nodes(&self) -> HashMap<NodeId, Node> {
        let mut nodes = HashMap::new();
        for inner_layer in self.service_handles.values() {
            for cell_node in inner_layer.cell.nodes().iter().all_except_local() {
                let node = cell_node.node().clone();
                nodes.insert(node.id().clone(), node);
            }
        }
        nodes
    }

    fn remove_handle(&mut self, cell_id: &CellId, layer: ServiceType) {
        self.service_handles.remove(&(cell_id.clone(), layer));
    }
}

pub(super) struct ServiceHandle {
    pub(super) cell: Cell,
    pub(super) in_sender: mpsc::Sender<InEvent>,
    pub(super) out_receiver: Option<mpsc::Receiver<OutEvent>>,
}

/// Handle taken by a Cell service to receive and send message for a given node &
/// cell.
pub struct Libp2pTransportServiceHandle {
    pub(super) cell_id: CellId,
    pub(super) service_type: ServiceType,
    pub(super) inner: Weak<RwLock<ServiceHandles>>,
    pub(super) sink: Option<mpsc::Sender<OutEvent>>,
    pub(super) stream: Option<mpsc::Receiver<InEvent>>,
    pub(super) handle: Handle,
}

impl TransportServiceHandle for Libp2pTransportServiceHandle {
    type Sink = SinkMapErr<mpsc::Sender<OutEvent>, fn(SendError) -> Error>;
    type Stream = mpsc::Receiver<InEvent>;

    fn on_started(&self) -> TransportHandleOnStart {
        Box::new(self.handle.on_set_started())
    }

    fn get_sink(&mut self) -> Self::Sink {
        self.sink
            .take()
            .expect("Sink was already consumed")
            .sink_map_err(|err| Error::Other(format!("Sink error: {}", err)))
    }

    fn get_stream(&mut self) -> Self::Stream {
        self.stream.take().expect("Stream was already consumed")
    }
}

impl Future for Libp2pTransportServiceHandle {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.handle.on_set_dropped().poll_unpin(cx)
    }
}

impl Drop for Libp2pTransportServiceHandle {
    fn drop(&mut self) {
        debug!(
            "Transport handle for cell {} and service {:?} got dropped. Removing it from transport",
            self.cell_id, self.service_type
        );

        // we have been dropped, we remove ourself from layers to communicate with
        if let Some(inner) = self.inner.upgrade() {
            if let Ok(mut inner) = inner.write() {
                inner.remove_handle(&self.cell_id, self.service_type);
            }
        }
    }
}
