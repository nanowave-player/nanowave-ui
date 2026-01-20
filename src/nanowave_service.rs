use serde_json::{json, Value};
use std::{
    sync::{Arc, Mutex},
    thread,
};
use tungstenite::{connect, Message};
use url::Url;


type MessageCallback = Arc<dyn Fn(Value) + Send + Sync>;

pub struct NanoWaveService {
    id: Arc<Mutex<i32>>,
    outgoing: Arc<Mutex<Vec<Value>>>,
    on_message: Arc<Mutex<Option<MessageCallback>>>,
}

impl NanoWaveService {
    pub fn new() -> Self {
        Self {
            id: Arc::new(Mutex::new(0)),
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

    fn increment_id(&self) -> i32 {
        // Lock the Mutex to safely access and modify the value
        let mut id = self.id.lock().unwrap(); // You could handle the lock failure with more sophisticated error handling
        *id += 1; // Increment the value inside the Mutex
        *id
    }

    /*
    fn send_message(&self, ) {
        let id = self.increment_id();
        let params = Value::new()
        self.outgoing.lock().unwrap().push(json!({
            "jsonrpc": "2.0",
            "method": "media_source_filter",
            "params": { "query": query },
            "id": id
        }));
    }
*/
    pub fn send_request(&self, method: &str, params: Value) {
        let id = self.increment_id();

        self.outgoing.lock().unwrap().push(
        json!({
                "jsonrpc": "2.0",
                "method": method,
                "params": params,
                "id": id
            })
        )
    }

    pub fn filter_media(&self, query: String) {
        self.send_request("media_source_filter", json!({
            "query": query
        }));
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
        if let Some(msg) = outgoing.lock().unwrap().pop() {
            let _ = socket.send(Message::Text(msg.to_string().into()));
        }

        match socket.read() {
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
                thread::sleep(std::time::Duration::from_millis(50));
            }
        }
    }
}

