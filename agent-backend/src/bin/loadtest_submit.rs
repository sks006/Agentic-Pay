//! A load‑testing binary that sends many `submitTransaction` requests with unique nonces.
//! Usage: cargo run --bin loadtest_submit -- --concurrency 50 --requests 1000

use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine;
use clap::Parser;
use jsonrpsee::core::client::ClientT;
use jsonrpsee::http_client::HttpClientBuilder;
use jsonrpsee::rpc_params;
use solana_sdk::transaction::VersionedTransaction;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::task::JoinSet;

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

    /// Agent public key (Base58)
    #[arg(long, default_value = "7Ec9tK1aP5A4pfnCbP1hJbX9qM7C2p8ZyzpXqgT5fLkK")]
    agent_pubkey: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Build JSON-RPC client
    let client = HttpClientBuilder::default()
        .build(&args.rpc_url)?;

    // A dummy transaction (just a placeholder; we just need a Base64 string)
    // In reality, you would create a real VersionedTransaction.
    // For load testing, we use a zero transaction (which will fail on-chain but
    // still exercises the state engine and RPC layer).
    let dummy_tx = VersionedTransaction::default();
    let tx_bytes = bincode::serialize(&dummy_tx)?;
    let tx_base64 = BASE64_STANDARD.encode(tx_bytes);

    let agent_pubkey = args.agent_pubkey;
    let total_requests = args.requests;
    let concurrency = args.concurrency;

    let nonce_counter = Arc::new(AtomicU64::new(1));

    println!("🚀 Starting load test on `submitTransaction`");
    println!("Concurrency: {}, Total requests: {}", concurrency, total_requests);
    println!("Using agent: {}", agent_pubkey);

    let start = Instant::now();
    let mut join_set = JoinSet::new();

    // Spawn workers
    let requests_per_worker = total_requests / concurrency;
    let remaining = total_requests % concurrency;

    for i in 0..concurrency {
        let client = client.clone();
        let tx_base64 = tx_base64.clone();
        let agent_pubkey = agent_pubkey.clone();
        let num_requests = requests_per_worker + if i < remaining { 1 } else { 0 };
        let counter = Arc::clone(&nonce_counter);

        join_set.spawn(async move {
            let mut results = Vec::with_capacity(num_requests);
            for _ in 0..num_requests {
                let nonce = counter.fetch_add(1, Ordering::Relaxed);
                let req_start = Instant::now();
                let response: Result<String, _> = client
                    .request(
                        "agent_submitTransaction",
                        rpc_params![tx_base64.clone(), agent_pubkey.clone(), nonce],
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
        for (duration, is_ok) in result.unwrap() {
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
    println!("Throughput: {:.0} req/s", total_requests as f64 / total_time.as_secs_f64());
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

    std::fs::write(
        output_path,
        serde_json::to_string_pretty(&output)?,
    )?;

    println!("✅ Results saved to {}", output_path);
    Ok(())
}
