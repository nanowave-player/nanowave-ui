use std::{
    sync::{Arc, Mutex, mpsc},
    thread,
};
use std::net::TcpStream;
use serde_json::{Value, json};
use tungstenite::{client, connect, Message};
use url::Url;

type MessageCallback = Arc<dyn Fn(Value) + Send + Sync>;

pub struct NanoWaveService {
    id: Mutex<i32>,
    outgoing_tx: mpsc::Sender<Value>,
    outgoing_rx: Mutex<Option<mpsc::Receiver<Value>>>,
    on_message: Arc<Mutex<Option<MessageCallback>>>,
}

impl NanoWaveService {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();

        Self {
            id: Mutex::new(0),
            outgoing_tx: tx,
            outgoing_rx: Mutex::new(Some(rx)),
            on_message: Arc::new(Mutex::new(None)),
        }
    }

    /// Start background threads (call once)
    pub fn run_in_background(&self) {




        let url = Url::parse("ws://localhost:8080").unwrap();
        let rx = self
            .outgoing_rx
            .lock()
            .unwrap()
            .take()
            .expect("run_in_background may only be called once");

        let (mut socket, _response) = connect(url.to_string()).expect("Failed to connect");


        let socket = Arc::new(Mutex::new(socket));

        let reader_socket = socket.clone();

        let writer_socket = socket.clone();
        thread::spawn(move || {
            while let Ok(msg) = rx.recv() {
                let mut socket = writer_socket.lock().unwrap();

                let message = msg.to_string();

                println!("rx message: {}", message.clone());
                socket
                    .send(Message::Text(message.into()))
                    .expect("Failed to send message");
                drop(socket);
                /*
                // let msg = r#"{"jsonrpc":"2.0","method":"media_source_filter","params":{"query":"2"},"id":"1"}"#;

                socket
                    .send(Message::Text(msg.into()))
                    .expect("Failed to send message");

                println!("Message sent");

                 */
            }


        });


        thread::spawn(move || {
            let mut socket = reader_socket.lock().unwrap();

/*            use url::Url;
            use tungstenite::{connect, Message};

            let (mut socket, response) = connect("ws://127.0.0.1:8080").unwrap();

            let r = socket.write(Message::Text(r#"{"jsonrpc": "2.0", "method":"media_source_filter", "params": {"query":"2"}, "id": "1"}"#.into()));

            println!("{:?}", r);
*/

            loop {
                println!("reader loop");

                let msg = socket.read().expect("Error reading message");
                println!("Received: {}", msg);
            }
        });
        /*
        thread::spawn(move || {

            use url::Url;
            use tungstenite::{connect, Message};

            let (mut socket, response) = connect("ws://127.0.0.1:8080").unwrap();

            let r = socket.write(Message::Text(r#"{"jsonrpc": "2.0", "method":"media_source_filter", "params": {"query":"2"}, "id": "1"}"#.into()));

            println!("{:?}", r);


            loop {
                let msg = socket.read().expect("Error reading message");
                println!("Received: {}", msg);
            }
        });
        */
/*
        let rx = self
            .outgoing_rx
            .lock()
            .unwrap()
            .take()
            .expect("run_in_background may only be called once");

        let on_message = self.on_message.clone();
        let url = Url::parse("ws://127.0.0.1:8080").unwrap();
        let (socket, _) = connect(url.to_string()).expect("WebSocket connect failed");

        let socket = Arc::new(Mutex::new(socket));
        let writer_socket = socket.clone();
        let reader_socket = socket.clone();

        thread::spawn(move || {
            // ---- Writer thread ----
            loop {
                println!("writer loop");
                while let Ok(msg) = rx.recv() {
                    println!("writer receive message: {}", msg);
                    let message = msg.to_string();
                    let mut ws = writer_socket.lock().unwrap();
                    let ws_send_result = ws.send(Message::Text(message.into()));
                    println!("writer send result: {:?}", ws_send_result);

                }
            }
        });


        thread::spawn(move || {
            // ---- Reader loop ----
            loop {
                let msg = {
                    let mut ws = reader_socket.lock().unwrap();
                    ws.read()
                };

                match msg {
                    Ok(Message::Text(txt)) => {
                        if let Ok(value) = serde_json::from_str::<Value>(&txt) {
                            if let Some(cb) = on_message.lock().unwrap().clone() {
                                slint::invoke_from_event_loop(move || {
                                    cb(value);
                                })
                                    .ok();
                            }
                        }
                    }
                    Ok(_) => {}
                    Err(_) => break,
                }
            }
        });
*/

    }

    fn next_id(&self) -> i32 {
        let mut id = self.id.lock().unwrap();
        *id += 1;
        *id
    }

    pub fn send_message(&self, method: &str, params: Value) {
        let msg = json!({
            "jsonrpc": "2.0",
            "id": self.next_id(),
            "method": method,
            "params": params
        });


        let result = self.outgoing_tx.send(msg.clone());
        println!("sending message {}, result: {:?}", msg, result);

    }

    // ---- API methods ----

    pub fn media_source_filter(&self, query: String) {
        self.send_message("media_source_filter", json!({ "query": query }));
    }

    pub fn media_source_find(&self, id: String) {
        self.send_message("media_source_find", json!({ "id": id }));
    }

    pub fn player_play_media(&self, id: String) {
        self.send_message("player_play_media", json!({ "id": id }));
    }

    pub fn on_message_received<F>(&self, callback: F)
    where
        F: Fn(Value) + Send + Sync + 'static,
    {
        *self.on_message.lock().unwrap() = Some(Arc::new(callback));
    }
}
