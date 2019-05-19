#![deny(bare_trait_objects)]

use wasm_bindgen::prelude::*;

#[macro_use]
use stdweb;

use stdweb::traits::*;
use stdweb::web::{WebSocket, SocketBinaryType};
use stdweb::web::event::{SocketMessageEvent, SocketOpenEvent, SocketMessageData};
use exocore_common::serialization::framed::{FrameBuilder, TypedFrame, TypedSliceFrame};
use exocore_common::serialization::protos::data_transport_capnp::envelope;

#[wasm_bindgen]
extern "C" {
    fn alert(msg: &str);

    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}


#[wasm_bindgen]
pub struct ExocoreClient {
    ws: WebSocket,
}

#[wasm_bindgen]
impl ExocoreClient {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<ExocoreClient, JsValue> {
        stdweb::initialize();

        let mut ws = WebSocket::new_with_protocols("ws://127.0.0.1:3341", &["exocore_websocket"]).unwrap();
        ws.set_binary_type(SocketBinaryType::ArrayBuffer);

        // SEE: https://github.com/koute/stdweb/blob/dff1e06086124fe79e3393a99ae8e2d424f5b2f1/examples/echo/src/main.rs
        ws.add_event_listener(move |event: SocketMessageEvent| {
            let data = Vec::from(event.data().into_array_buffer().unwrap());
            let frame = TypedSliceFrame::<envelope::Owned>::new(&data).unwrap();
            let envelope_reader: envelope::Reader = frame.get_typed_reader().unwrap();
            log(&format!("Got message> {}", String::from_utf8_lossy(envelope_reader.get_data().unwrap())));
        });

        ws.add_event_listener(move |event: SocketOpenEvent| {
            log("Connected");
        });

        log("Websocket connected");
        //alert("Hello world !");

        Ok(ExocoreClient {
            ws
        })
    }

    #[wasm_bindgen]
    pub fn send(&self, text: &str) {
        let mut frame_builder = FrameBuilder::<envelope::Owned>::new();
        let mut envelope_builder: envelope::Builder = frame_builder.get_builder_typed();
        envelope_builder.set_data(text.as_bytes());
        let frame = frame_builder.as_owned_unsigned_framed().unwrap();

        self.ws.send_bytes(frame.frame_data()).unwrap();
    }
}


impl Drop for ExocoreClient {
    fn drop(&mut self) {
        log("Got dropped");
        // TODO: Close connection ?
    }
}
