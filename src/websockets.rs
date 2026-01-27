use std::sync::mpsc;
use std::thread;
use tokio::runtime::Runtime;

use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio_tungstenite::connect_async;
use tungstenite::Message;
use url::Url;
pub fn start_websocket_runtime(url_string: String,) -> (
    
    UnboundedSender<String>,
    UnboundedReceiver<String>,
) {
    let (outgoing_tx, outgoing_rx) = tokio::sync::mpsc::unbounded_channel();
    let (incoming_tx, incoming_rx) = tokio::sync::mpsc::unbounded_channel();

    thread::spawn(move || {
        let rt = Runtime::new().unwrap();
        rt.block_on(websocket_task(url_string, outgoing_rx, incoming_tx));
    });

    (outgoing_tx, incoming_rx)
}


pub async fn websocket_task(
    url_string: String,
    mut outgoing_rx: UnboundedReceiver<String>,
    incoming_tx: UnboundedSender<String>,
) {
    // url_string: "wss://echo.websocket.events"
    let url = Url::parse(url_string.as_str()).unwrap();
    let (ws_stream, _) = connect_async(url.to_string()).await.expect("Failed to connect");

    let (mut write, mut read) = ws_stream.split();

    // Read task
    let read_task = tokio::spawn(async move {
        while let Some(msg) = read.next().await {
            if let Ok(Message::Text(text)) = msg {
                let _ = incoming_tx.send(text.to_string());
            }
        }
    });

    // Write task
    let write_task = tokio::spawn(async move {
        while let Some(msg) = outgoing_rx.recv().await {
            let _ = write.send(Message::Text(msg.into())).await;
        }
    });

    let _ = tokio::join!(read_task, write_task);
}