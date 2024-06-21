use std::time::{Duration, Instant};

use tiny_keccak::{Hasher, Keccak};
use tokio_util::sync::CancellationToken;

const TIME_LIMIT: Duration = Duration::from_millis(800);

pub async fn find_nonce_parallel(
    random: u64,
    leading_zeros: u8,
    token: CancellationToken,
) -> Option<u64> {
    let (result_tx, mut result_rx) = tokio::sync::mpsc::channel(1);

    let num_cores = num_cpus::get();
    for _ in 0..num_cores {
        let token_clone = token.clone();
        let result_tx = result_tx.clone();
        tokio::spawn(async move {
            let nonce = find_nonce(random, leading_zeros, token_clone);
            if let Some(nonce) = nonce {
                let _ = result_tx.send(nonce).await;
            }
        });
    }

    let result = result_rx.recv().await;
    token.cancel();
    return result;
}

pub fn find_nonce(random: u64, leading_zeros: u8, token: CancellationToken) -> Option<u64> {
    let start = Instant::now();
    let mut nonce: u64;
    let target_zeros = leading_zeros as usize;

    loop {
        // Check if we should cancel
        if token.is_cancelled() {
            // let duration = start.elapsed();
            // println!("Task was cancelled after {:?}", duration);
            return None; // Task was cancelled
        }

        nonce = rand::random();
        let mut data = Vec::with_capacity(16);
        data.extend_from_slice(&random.to_le_bytes());
        data.extend_from_slice(&nonce.to_le_bytes());
        let hash = keccak256(&data);
        if count_leading_zeros(&hash) == target_zeros {
            let duration = start.elapsed();
            // println!("Found valid nonce in {:?}", duration);
            return if duration > TIME_LIMIT {
                None
            } else {
                Some(nonce) // Found a valid nonce
            };
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

fn count_leading_zeros(hash: &[u8]) -> usize {
    hash.iter()
        .flat_map(|byte| (0..8).rev().map(move |i| (byte >> i) & 1))
        .take_while(|&bit| bit == 0)
        .count()
}
