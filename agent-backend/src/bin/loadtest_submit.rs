//! A load‑testing binary that sends many `submitTransaction` requests with unique nonces.
//! Usage: cargo run --bin loadtest_submit -- --concurrency 50 --requests 1000

use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine;
use clap::Parser;
use jsonrpsee::core::client::ClientT;
use jsonrpsee::http_client::HttpClientBuilder;
use jsonrpsee::rpc_params;
use solana_sdk::transaction::VersionedTransaction;
use std::time::{Duration, Instant};
use tokio::task::JoinSet;

#[derive(Clone, Debug)]
pub struct RpcRequestPayload {
    pub agent_pubkey_hex: String,
    pub nonce: u64,
    pub tx_base64: String,
}

#[derive(Clone, Debug)]
pub struct PreComputedLoadMatrix {
    pub payloads: Vec<RpcRequestPayload>,
}

pub trait PayloadMatrixGenerator {
    type MatrixFault;

    /// Allocates 1,000 mathematically distinct agent/nonce pairs into memory
    /// BEFORE the benchmark timer initiates.
    fn construct_matrix(
        total_requests: usize,
    ) -> Result<PreComputedLoadMatrix, Self::MatrixFault>;
}

pub struct DeterministicPayloadGenerator;

impl PayloadMatrixGenerator for DeterministicPayloadGenerator {
    type MatrixFault = anyhow::Error;

    fn construct_matrix(
        total_requests: usize,
    ) -> Result<PreComputedLoadMatrix, Self::MatrixFault> {
        let dummy_tx = VersionedTransaction::default();
        let tx_bytes = bincode::serialize(&dummy_tx)?;
        let tx_base64 = BASE64_STANDARD.encode(tx_bytes);

        let mut payloads = Vec::with_capacity(total_requests);

        for i in 0..total_requests {
            let mut key_bytes = [0u8; 32];
            let idx_bytes = ((i as u64) + 1).to_be_bytes();
            key_bytes[24..32].copy_from_slice(&idx_bytes);

            let agent_pubkey_hex = hex::encode(key_bytes);
            let nonce = (i as u64) + 1;

            payloads.push(RpcRequestPayload {
                agent_pubkey_hex,
                nonce,
                tx_base64: tx_base64.clone(),
            });
        }

        Ok(PreComputedLoadMatrix { payloads })
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Number of concurrent tasks
    #[arg(short, long, default_value_t = 50)]
    concurrency: usize,

    /// Total number of requests to send
    #[arg(short, long, default_value_t = 1000)]
    requests: usize,

    /// RPC server URL
    #[arg(long, default_value = "http://localhost:8545")]
    rpc_url: String,

    /// Agent public key (optional fallback / override)
    #[arg(long, default_value = "")]
    agent_pubkey: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Build JSON-RPC client
    let client = HttpClientBuilder::default().build(&args.rpc_url)?;

    let total_requests = args.requests;
    let concurrency = args.concurrency;

    println!(
        "📦 Pre-computing deterministic payload matrix ({} distinct agent/nonce pairs)...",
        total_requests
    );
    let matrix = DeterministicPayloadGenerator::construct_matrix(total_requests)?;

    // Partition payloads across workers before starting the timer to avoid in-thread allocations
    let mut worker_payloads_list: Vec<Vec<RpcRequestPayload>> = (0..concurrency)
        .map(|_| Vec::new())
        .collect();

    for (i, payload) in matrix.payloads.into_iter().enumerate() {
        worker_payloads_list[i % concurrency].push(payload);
    }

    println!("🚀 Starting load test on `submitTransaction`");
    println!("Concurrency: {}, Total requests: {}", concurrency, total_requests);
    println!("Matrix size: {} pre-computed payloads", total_requests);

    // Start benchmark timer ONLY after the payload vector is fully populated and partitioned
    let start = Instant::now();
    let mut join_set = JoinSet::new();

    for worker_payloads in worker_payloads_list {
        let client = client.clone();
        join_set.spawn(async move {
            let mut results = Vec::with_capacity(worker_payloads.len());
            for payload in worker_payloads {
                let req_start = Instant::now();
                let response: Result<String, _> = client
                    .request(
                        "agent_submitTransaction",
                        rpc_params![payload.tx_base64, payload.agent_pubkey_hex, payload.nonce],
                    )
                    .await;
                let req_duration = req_start.elapsed();
                results.push((req_duration, response.is_ok()));
            }
            results
        });
    }

    // Collect results
    let mut all_latencies = Vec::with_capacity(total_requests);
    let mut success_count = 0;
    let mut error_count = 0;

    while let Some(result) = join_set.join_next().await {
        for (duration, is_ok) in result? {
            all_latencies.push(duration);
            if is_ok {
                success_count += 1;
            } else {
                error_count += 1;
            }
        }
    }

    let total_time = start.elapsed();

    // Compute stats
    if all_latencies.is_empty() {
        println!("No requests were executed.");
        return Ok(());
    }
    all_latencies.sort();
    let p50 = all_latencies[all_latencies.len() / 2];
    let p95 = all_latencies[(all_latencies.len() * 95) / 100];
    let p99 = all_latencies[(all_latencies.len() * 99) / 100];
    let avg = all_latencies.iter().sum::<Duration>() / all_latencies.len() as u32;

    println!("\n📊 Results:");
    println!("Total time: {:?}", total_time);
    println!("Requests: {}", total_requests);
    println!("Success: {}", success_count);
    println!("Errors: {}", error_count);
    println!(
        "Throughput: {:.0} req/s",
        total_requests as f64 / total_time.as_secs_f64()
    );
    println!("Latencies:");
    println!("  Avg: {:?}", avg);
    println!("  P50: {:?}", p50);
    println!("  P95: {:?}", p95);
    println!("  P99: {:?}", p99);

    // Save to JSON file for daily tracking
    let output = serde_json::json!({
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "type": "submit_transaction",
        "concurrency": concurrency,
        "requests": total_requests,
        "success": success_count,
        "errors": error_count,
        "throughput_rps": total_requests as f64 / total_time.as_secs_f64(),
        "latency_avg_ms": avg.as_secs_f64() * 1000.0,
        "latency_p50_ms": p50.as_secs_f64() * 1000.0,
        "latency_p95_ms": p95.as_secs_f64() * 1000.0,
        "latency_p99_ms": p99.as_secs_f64() * 1000.0,
    });

    let _ = std::fs::create_dir_all("benchmarks/results");
    let _ = std::fs::create_dir_all("../benchmarks/results");
    let output_path = if std::path::Path::new("benchmarks/results").exists() {
        "benchmarks/results/latest_submit_load.json"
    } else {
        "../benchmarks/results/latest_submit_load.json"
    };

    std::fs::write(output_path, serde_json::to_string_pretty(&output)?)?;

    println!("✅ Results saved to {}", output_path);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_construct_matrix_uniqueness() {
        let matrix = DeterministicPayloadGenerator::construct_matrix(1000).unwrap();
        assert_eq!(matrix.payloads.len(), 1000);

        let mut pubkeys = HashSet::new();
        let mut nonces = HashSet::new();

        for payload in matrix.payloads {
            assert_eq!(payload.agent_pubkey_hex.len(), 64);
            assert!(pubkeys.insert(payload.agent_pubkey_hex));
            assert!(nonces.insert(payload.nonce));
        }
    }
}
