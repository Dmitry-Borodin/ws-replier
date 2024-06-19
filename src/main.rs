use futures_util::{SinkExt, StreamExt};
use tiny_keccak::{Hasher, Keccak};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use std::convert::TryInto;

const SERVER_URL: &str = "ws://34.159.18.133:22430/";

#[tokio::main]
async fn main() {
    env_logger::init();

    let (ws_stream, _) = connect_async(SERVER_URL).await.expect("Can't connect");
    let (mut write, mut read) = ws_stream.split();

    while let Some(msg) = read.next().await {
        let message = msg.expect("Failed to read message");

        // println!("Received: {:?}", message.into_data());

        match message {
            msg @ Message::Text(_) => {
                println!("Text message: {:?}", msg.into_text());
            }
            msg @ Message::Binary(_) => {
                let data = msg.into_data();
                if data.len() == 9 {
                    let response = build_response(data);
                    write.send(Message::binary(response)).await.expect("Failed to send response");
                } else if data.len() == 4 {
                    let score = u32::from_be_bytes(data.try_into().unwrap());
                    println!("Score: {}", score);
                }
            }
            Message::Ping(_) | Message::Pong(_) | Message::Close(_) | Message::Frame(_) => {}
        }
    }
}

fn build_response(challange: Vec<u8>) -> Vec<u8> {
        let random = u64::from_le_bytes(challange[0..8].try_into().unwrap());
        let leading_zeros = challange[8];

        let nonce = find_nonce(random, leading_zeros);

        let mut response = Vec::with_capacity(16);
        response.extend(&random.to_le_bytes());
        response.extend(&nonce.to_le_bytes());

        return response;
}

fn keccak256(data: &[u8]) -> Vec<u8> {
    let mut keccak = Keccak::v256();
    keccak.update(data);
    let mut result = vec![0u8; 32];
    keccak.finalize(&mut result);
    result
}

fn find_nonce(random: u64, leading_zeros: u8) -> u64 {
    let mut nonce: u64;
    let target_zeros = leading_zeros as usize;

    loop {
        nonce = rand::random();
        let mut data = Vec::with_capacity(16);
        data.extend_from_slice(&random.to_le_bytes());
        data.extend_from_slice(&nonce.to_le_bytes());
        let hash = keccak256(&data);
        if count_leading_zeros(&hash) == target_zeros {
            break;
        }
    }
    nonce
}

fn count_leading_zeros(hash: &[u8]) -> usize {
    hash.iter().flat_map(|byte| (0..8).rev().map(move |i| (byte >> i) & 1)).take_while(|&bit| bit == 0).count()
}
