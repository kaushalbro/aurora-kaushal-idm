use super::common::ProcessMetrics;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub fn linux_process_metrics() -> ProcessMetrics {
    let mut rss_bytes = 0u64;

    if let Ok(file) = File::open("/proc/self/status") {
        let reader = BufReader::new(file);
        for line in reader.lines().flatten() {
            if line.starts_with("VmRSS:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(kb) = parts[1].parse::<u64>() {
                        rss_bytes = kb * 1024;
                    }
                }
                break;
            }
        }
    }

    ProcessMetrics {
        resident_memory_bytes: rss_bytes,
        cpu_usage_pct: 0.0,
    }
}
