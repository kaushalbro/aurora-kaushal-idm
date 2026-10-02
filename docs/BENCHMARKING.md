# AURORA Benchmarking Methodology & External Comparisons

This document specifies the scientific, fair, and reproducible benchmarking methodology for comparing the AURORA download engine against established download managers (IDM, FDM, aria2, and standard browsers).

---

## 1. Principles of Fair External Benchmarking

To ensure objective and unbiased performance evaluation:

1. **Identical Test Environment**:
   - Same physical machine and network interface (Gigabit / 10 GbE / Wi-Fi).
   - Same destination storage drive (NVMe SSD or high-speed RAM-disk to isolate network performance from disk bottlenecks).
   - Same test period to minimize external ISP transit congestion shifts.
2. **Identical Remote Target**:
   - Same high-bandwidth test file (e.g., 1 GB or 10 GB binary dataset on Hetzner, AWS CloudFront, or Cloudflare R2).
   - Same HTTP protocol version (HTTP/1.1 or HTTP/2).
3. **Repeated Trials with Statistical Reporting**:
   - Minimum $N \ge 5$ iterations per tool.
   - Report **Median**, **Mean**, **Standard Deviation**, and **p95 Completion Time**.
   - Do **NOT** cherry-pick favorable single runs.

---

## 2. Automated Multi-Scheduler Benchmark Suite

AURORA includes an integrated benchmarking harness:

```bash
cargo run --release --bin aurora -- benchmark <URL> \
  --runs 5 \
  --format json \
  --report-file ./benchmark_results.json
```

This will automatically execute multi-trial runs across:
1. `SingleStream`
2. `FixedSegmentation` (8 connections)
3. `LargestSegment` (IDM-style dynamic 50/50 splitting)
4. `AuroraEct` (Expected Completion Time scheduling with rate-proportional splitting)

---

## 3. Comparison Metrics

| Metric | Description | Unit |
| :--- | :--- | :--- |
| **Completion Time** | Total wall-clock time from URL trigger to whole-file verification | Seconds ($s$) |
| **Average Throughput** | Total file bytes divided by total elapsed seconds | Mbps or MB/s |
| **Throughput Variance** | Stability of stream speed over time | $\sigma$ (MB/s) |
| **Peak Throughput** | Maximum observed 1-second burst rate | MB/s |
| **Retry Overhead** | Total redundant or retried bytes transferred | Bytes |
| **Memory Footprint** | Peak Resident Set Size (RSS) during active download | MB |
| **CPU Utilization** | Average user + system CPU percentage | $\%$ |

---

## 4. Local Mock Server Testing (No Internet Required)

To evaluate scheduler performance without public network variability:

```bash
# Terminal 1: Launch Mock Test Server with simulated latency & rate-limited streams
cargo run --release --bin aurora -- test-server --size-mb 100 --rate-limit-mb 20 --latency-ms 20

# Terminal 2: Run benchmark suite against the mock server
cargo run --release --bin aurora -- benchmark http://127.0.0.1:8080/testfile.bin --runs 3
```
