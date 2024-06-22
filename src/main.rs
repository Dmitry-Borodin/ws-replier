use std::convert::TryInto;
use std::sync::Arc;

use futures_util::SinkExt;
use futures_util::stream::StreamExt;
use tokio::sync::Mutex;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use tokio_util::sync::CancellationToken;

use ws_replier::find_nonce;

const SERVER_URL: &str = "ws://34.159.18.133:22430/";
const DEBUGGING: bool = true;

#[tokio::main]
async fn main() {
    env_logger::init();

    let (ws_stream, _) = connect_async(SERVER_URL).await.expect("Can't connect");
    let (write, mut read) = ws_stream.split();

    let write = Arc::new(Mutex::new(write));

    // Token for task cancellation
    let cancel_token = Arc::new(Mutex::new(CancellationToken::new()));

    //main loop of websocket messages
    while let Some(msg) = read.next().await {
        let message = msg.expect("Failed to read message");

        match message {
            msg @ Message::Text(_) => {
                println!("Text message: {:?}", msg.into_text());
            }
            msg @ Message::Binary(_) => {
                let data = msg.into_data();
                if data.len() == 9 {
                    if DEBUGGING {
                        println!("Received request: {:?}", data.clone());
                    }
                    let random = u64::from_le_bytes(data[0..8].try_into().unwrap());
                    let leading_zeros = data[8];

                    // Cancel the previous task
                    let token = Arc::clone(&cancel_token);
                    let mut token = token.lock().await;
                    token.cancel();

                    // Create a new cancellation token for the new task
                    *token = CancellationToken::new();

                    let token_clone = token.clone();
                    let write_clone = write.clone();

                    tokio::spawn(async move {
                        let nonce = find_nonce(random, leading_zeros, token_clone.clone());
                        if let Some(nonce) = nonce {
                            let mut response = Vec::with_capacity(16);
                            response.extend(&random.to_le_bytes());
                            response.extend(&nonce.to_le_bytes());

                            if token_clone.is_cancelled() {
                                return; // Task was cancelled
                            }
                            if DEBUGGING {
                                println!("response for request: {:?}", data.clone());
                            }
                            let mut write = write_clone.lock().await;
                            write
                                .send(Message::binary(response))
                                .await
                                .expect("Failed to send response");
                        }
                    });
                } else if data.len() == 4 {
                    let score = u32::from_be_bytes(data.try_into().unwrap());
                    println!("Score: {}", score);
                } else {
                    println!("Received unexpected request: {:?}", data);
                }
            }
            Message::Ping(_) | Message::Pong(_) | Message::Close(_) | Message::Frame(_) => {
                if DEBUGGING {
                    println!("Received unexpected message: {:?}", message);
                }
            }
        }
    }
}
