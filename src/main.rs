use std::convert::TryInto;
use std::sync::Arc;

use async_channel::bounded;
use futures_util::SinkExt;
use futures_util::stream::StreamExt;
use tiny_keccak::{Hasher, Keccak};
use tokio::sync::{Mutex, watch};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use tokio_util::sync::CancellationToken;

const SERVER_URL: &str = "ws://34.159.18.133:22430/";
const DEBUGGING: bool = true;

#[tokio::main]
async fn main() {
    env_logger::init();

    let (ws_stream, _) = connect_async(SERVER_URL).await.expect("Can't connect");
    let (write, mut read) = ws_stream.split();

    let write = Arc::new(Mutex::new(write));
    let (tx, rx) = watch::channel((0u64, 0u8));
    let (cancel_tx, _) = bounded::<(u64, u8)>(1);

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

                    // Send the new challenge to the channel
                    tx.send((random, leading_zeros)).unwrap();

                    // Cancel the previous task
                    let token = Arc::clone(&cancel_token);
                    let mut token = token.lock().await;
                    token.cancel();

                    // Create a new cancellation token for the new task
                    *token = CancellationToken::new();

                    let token_clone = token.clone();
                    let write_clone = write.clone();

                    tokio::spawn(async move {
                        let nonce = find_nonce(random, leading_zeros, token_clone.clone()).await;
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

fn keccak256(data: &[u8]) -> Vec<u8> {
    let mut keccak = Keccak::v256();
    keccak.update(data);
    let mut result = vec![0u8; 32];
    keccak.finalize(&mut result);
    result
}

async fn find_nonce(random: u64, leading_zeros: u8, token: CancellationToken) -> Option<u64> {
    let mut nonce: u64;
    let target_zeros = leading_zeros as usize;

    loop {
        // Check if we should cancel
        if token.is_cancelled() {
            return None; // Task was cancelled
        }

        nonce = rand::random();
        let mut data = Vec::with_capacity(16);
        data.extend_from_slice(&random.to_le_bytes());
        data.extend_from_slice(&nonce.to_le_bytes());
        let hash = keccak256(&data);
        if count_leading_zeros(&hash) == target_zeros {
            return Some(nonce); // Found a valid nonce
        }

        // Yield to the Tokio runtime to prevent blocking
        // tokio::task::yield_now().await;
    }
}

fn count_leading_zeros(hash: &[u8]) -> usize {
    hash.iter()
        .flat_map(|byte| (0..8).rev().map(move |i| (byte >> i) & 1))
        .take_while(|&bit| bit == 0)
        .count()
}
