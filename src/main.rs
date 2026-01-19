use futures_util::SinkExt;
use serde_json::json;
use slint::SharedString;
use tokio_tungstenite::tungstenite::Message;
use url::Url;

slint::include_modules!();

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ui = MainWindow::new()?;

    let ui_handle = ui.as_weak();

    ui.on_play_requested(move |media_id: SharedString| {
        let media_id = media_id.to_string();

        // Spawn async task so UI thread stays responsive
        tokio::spawn(async move {
            if let Err(e) = send_play_request(media_id).await {
                eprintln!("Failed to send play request: {}", e);
            }
        });
    });

    ui.run()?;
    Ok(())
}

async fn send_play_request(media_id: String) -> Result<(), Box<dyn std::error::Error>> {
    let url = Url::parse("ws://127.0.0.1:8080")?;
    let (mut ws, _) = tokio_tungstenite::connect_async(url.to_string()).await?;

    let request = json!({
        "jsonrpc": "2.0",
        "method": "player_play_media",
        "params": {
            "id": media_id
        },
        "id": 1
    });

    ws.send(Message::Text(request.to_string().into())).await?;

    // Optional: read response (can be omitted)
    /*
    if let Some(msg) = ws.next().await {
        if let Ok(Message::Text(txt)) = msg {
            println!("Server response: {}", txt);
        }
    }
    */
    Ok(())
}
