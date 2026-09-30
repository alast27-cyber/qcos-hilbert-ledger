use std::env;
use std::error::Error;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::windows::named_pipe::{ClientOptions, NamedPipeClient};

const PIPE_NAME: &str = r"\\.\pipe\qcos_hilbert_ledger";

// Default Benchmark Constants
const DEFAULT_PERSISTENT_REQUESTS: usize = 10_000;
const DEFAULT_PERSISTENT_CONCURRENCY: usize = 50;

const DEFAULT_CHURN_REQUESTS: usize = 10_000;
const DEFAULT_CHURN_CONCURRENCY: usize = 50;

async fn connect_with_retry(pipe_name: &str, max_retries: u32) -> Result<NamedPipeClient, std::io::Error> {
    let mut attempt = 0;
    loop {
        match ClientOptions::new().open(pipe_name) {
            Ok(client) => return Ok(client),
            Err(e) => {
                attempt += 1;
                if attempt >= max_retries {
                    return Err(e);
                }
                // Exponential backoff capped at 2ms max sleep per attempt
                let backoff_us = (100 * (1 << (attempt - 1))).min(2000);
                tokio::time::sleep(Duration::from_micros(backoff_us as u64)).await;
            }
        }
    }
}

fn get_env_or_default(var_name: &str, default_val: usize) -> usize {
    env::var(var_name)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default_val)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let persistent_requests = get_env_or_default("BENCH_TOTAL_REQUESTS", DEFAULT_PERSISTENT_REQUESTS);
    let persistent_concurrency = get_env_or_default("BENCH_CONCURRENCY", DEFAULT_PERSISTENT_CONCURRENCY);
    let churn_requests = get_env_or_default("CHURN_TOTAL_REQUESTS", DEFAULT_CHURN_REQUESTS);
    let churn_concurrency = get_env_or_default("CHURN_CONCURRENCY", DEFAULT_CHURN_CONCURRENCY);

    println!("====================================================");
    println!("     QCOS HILBERT LEDGER BENCHMARK HARNESS          ");
    println!("====================================================");
    println!(" Pipe Target         : {}", PIPE_NAME);
    println!(" Persistent Pass     : {} requests across {} workers", persistent_requests, persistent_concurrency);
    println!(" High Churn Pass     : {} cycles across {} workers", churn_requests, churn_concurrency);
    println!("====================================================\n");

    println!("[Pass 1/2] Running Persistent Connection Pool Stress...");
    run_persistent_pool_test(persistent_requests, persistent_concurrency).await?;

    println!();

    println!("[Pass 2/2] Running High Churn Connection Stress...");
    run_high_churn_test(churn_requests, churn_concurrency).await?;

    println!("\n====================================================");
    println!(" Benchmark Execution Complete.");
    println!("==================================================== ");

    Ok(())
}

async fn run_persistent_pool_test(total_requests: usize, concurrency: usize) -> Result<(), Box<dyn Error>> {
    let requests_per_worker = total_requests / concurrency;
    let successful_ops = Arc::new(AtomicUsize::new(0));
    let failed_ops = Arc::new(AtomicUsize::new(0));

    let start_time = Instant::now();
    let mut handles = Vec::new();

    for _ in 0..concurrency {
        let success = Arc::clone(&successful_ops);
        let failure = Arc::clone(&failed_ops);

        let handle = tokio::spawn(async move {
            let mut client = match connect_with_retry(PIPE_NAME, 10).await {
                Ok(c) => c,
                Err(_) => {
                    failure.fetch_add(requests_per_worker, Ordering::SeqCst);
                    return;
                }
            };

            let payload = b"PING_HILBERT_LEDGER_PERSISTENT_PAYLOAD";
            let mut read_buf = [0u8; 64];

            for _ in 0..requests_per_worker {
                if client.write_all(payload).await.is_err() || client.flush().await.is_err() {
                    failure.fetch_add(1, Ordering::SeqCst);
                    continue;
                }

                match client.read(&mut read_buf).await {
                    Ok(n) if n > 0 => { success.fetch_add(1, Ordering::SeqCst); }
                    _ => { failure.fetch_add(1, Ordering::SeqCst); }
                }
            }
        });
        handles.push(handle);
    }

    for handle in handles { let _ = handle.await; }

    let elapsed = start_time.elapsed();
    let succ = successful_ops.load(Ordering::SeqCst);
    let fail = failed_ops.load(Ordering::SeqCst);
    let total = succ + fail;
    let ops_per_sec = (succ as f64) / elapsed.as_secs_f64();

    println!(" -> Completed in      : {:.2?}", elapsed);
    println!(" -> Success Rate      : {}/{} ({:.2}%)", succ, total, (succ as f64 / total as f64) * 100.0);
    println!(" -> Throughput        : {:.2} req/sec", ops_per_sec);

    Ok(())
}

async fn run_high_churn_test(total_requests: usize, concurrency: usize) -> Result<(), Box<dyn Error>> {
    let requests_per_worker = total_requests / concurrency;
    let successful_ops = Arc::new(AtomicUsize::new(0));
    let failed_ops = Arc::new(AtomicUsize::new(0));

    let start_time = Instant::now();
    let mut handles = Vec::new();

    for _ in 0..concurrency {
        let success = Arc::clone(&successful_ops);
        let failure = Arc::clone(&failed_ops);

        let handle = tokio::spawn(async move {
            let payload = b"PING_CHURN_PAYLOAD";
            let mut read_buf = [0u8; 64];

            for _ in 0..requests_per_worker {
                let mut client = match connect_with_retry(PIPE_NAME, 20).await {
                    Ok(c) => c,
                    Err(_) => {
                        failure.fetch_add(1, Ordering::SeqCst);
                        continue;
                    }
                };

                if client.write_all(payload).await.is_ok() && client.flush().await.is_ok() {
                    match client.read(&mut read_buf).await {
                        Ok(n) if n > 0 => { success.fetch_add(1, Ordering::SeqCst); }
                        _ => { failure.fetch_add(1, Ordering::SeqCst); }
                    }
                } else {
                    failure.fetch_add(1, Ordering::SeqCst);
                }
            }
        });
        handles.push(handle);
    }

    for handle in handles { let _ = handle.await; }

    let elapsed = start_time.elapsed();
    let succ = successful_ops.load(Ordering::SeqCst);
    let fail = failed_ops.load(Ordering::SeqCst);
    let total = succ + fail;
    let ops_per_sec = (succ as f64) / elapsed.as_secs_f64();

    println!(" -> Completed in      : {:.2?}", elapsed);
    println!(" -> Success Rate      : {}/{} ({:.2}%)", succ, total, (succ as f64 / total as f64) * 100.0);
    println!(" -> Connection Rate   : {:.2} conn/sec", ops_per_sec);

    Ok(())
}