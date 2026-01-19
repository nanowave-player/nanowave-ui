use serde_json::{json, Value};
use std::{
    sync::{Arc, Mutex},
    thread,
};
use tungstenite::{connect, Message};
use url::Url;


type MessageCallback = Arc<dyn Fn(Value) + Send + Sync>;

pub struct NanoWaveService {
    outgoing: Arc<Mutex<Vec<Value>>>,
    on_message: Arc<Mutex<Option<MessageCallback>>>,
}

impl NanoWaveService {
    pub fn new() -> Self {
        Self {
            outgoing: Arc::new(Mutex::new(Vec::new())),
            on_message: Arc::new(Mutex::new(None)),
        }
    }

    /// Start background thread (call once)
    pub fn run_in_background(&self) {
        let outgoing = self.outgoing.clone();
        let on_message = self.on_message.clone();

        thread::spawn(move || {
            run_service_loop(outgoing, on_message);
        });
    }

    pub fn play_media(&self, id: String) {
        self.outgoing.lock().unwrap().push(json!({
            "jsonrpc": "2.0",
            "method": "player_play_media",
            "params": { "id": id },
            "id": 1
        }));
    }

    pub fn on_message_received<F>(&self, callback: F)
    where
        F: Fn(Value) + Send + Sync + 'static,
    {
        *self.on_message.lock().unwrap() = Some(Arc::new(callback));
    }
}

fn run_service_loop(
    outgoing: Arc<Mutex<Vec<Value>>>,
    on_message: Arc<Mutex<Option<MessageCallback>>>,
) {
    let url = Url::parse("ws://127.0.0.1:8080").unwrap();
    let (mut socket, _) = connect(url.to_string()).expect("WebSocket connect failed");

    loop {
        // Send queued messages
        if let Some(msg) = outgoing.lock().unwrap().pop() {
            let _ = socket.write_message(Message::Text(msg.to_string().into()));
        }

        match socket.read_message() {
            Ok(Message::Text(txt)) => {
                if let Ok(value) = serde_json::from_str::<Value>(&txt) {
                    // 🔑 Clone callback out of mutex
                    let callback = {
                        on_message.lock().unwrap().clone()
                    };

                    if let Some(cb) = callback {
                        let value = value.clone();

                        slint::invoke_from_event_loop(move || {
                            cb(value);
                        })
                            .unwrap();
                    }
                }
            }
            Ok(_) => {}
            Err(_) => {
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
        }
    }
}

