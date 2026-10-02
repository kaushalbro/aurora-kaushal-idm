use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationResult {
    pub scheduler_type: String,
    pub total_file_bytes: u64,
    pub worker_speeds_mbps: Vec<f64>,
    pub completion_time_seconds: f64,
    pub theoretical_optimal_seconds: f64,
    pub efficiency_pct: f64,
    pub total_splits: usize,
    pub worker_idle_time_seconds: f64,
}

/// Discrete-event mathematical simulator comparing download scheduling algorithms under non-uniform worker conditions.
pub struct SchedulerSimulator {
    pub file_size_bytes: u64,
    pub worker_speeds_bytes_per_sec: Vec<f64>,
    pub min_segment_size: u64,
}

impl SchedulerSimulator {
    pub fn new(file_size_bytes: u64, worker_speeds_mb_per_sec: &[f64]) -> Self {
        Self {
            file_size_bytes,
            worker_speeds_bytes_per_sec: worker_speeds_mb_per_sec
                .iter()
                .map(|&mb| mb * 1024.0 * 1024.0)
                .collect(),
            min_segment_size: 1024 * 1024, // 1 MB
        }
    }

    pub fn run_all_simulations(&self) -> Vec<SimulationResult> {
        vec![
            self.simulate_single_stream(),
            self.simulate_fixed(),
            self.simulate_largest_segment(),
            self.simulate_aurora_ect(),
        ]
    }

    pub fn theoretical_optimal(&self) -> f64 {
        let aggregate_rate: f64 = self.worker_speeds_bytes_per_sec.iter().sum();
        if aggregate_rate > 0.0 {
            self.file_size_bytes as f64 / aggregate_rate
        } else {
            0.0
        }
    }

    /// Simulation A: Single stream baseline (uses fastest worker or Worker 0).
    pub fn simulate_single_stream(&self) -> SimulationResult {
        let rate = self.worker_speeds_bytes_per_sec[0].max(1024.0);
        let completion = self.file_size_bytes as f64 / rate;
        let opt = self.theoretical_optimal();

        SimulationResult {
            scheduler_type: "Single Stream".to_string(),
            total_file_bytes: self.file_size_bytes,
            worker_speeds_mbps: self.worker_speeds_bytes_per_sec.iter().map(|b| b / (1024.0 * 1024.0)).collect(),
            completion_time_seconds: completion,
            theoretical_optimal_seconds: opt,
            efficiency_pct: (opt / completion * 100.0).clamp(0.0, 100.0),
            total_splits: 0,
            worker_idle_time_seconds: 0.0,
        }
    }

    /// Simulation B: Fixed static segmentation (equal byte slices, straggler sets completion time).
    pub fn simulate_fixed(&self) -> SimulationResult {
        let num_workers = self.worker_speeds_bytes_per_sec.len();
        let chunk_size = self.file_size_bytes / num_workers as u64;

        let mut max_time = 0.0f64;
        let mut total_idle = 0.0f64;

        let mut times = Vec::new();
        for &rate in &self.worker_speeds_bytes_per_sec {
            let t = chunk_size as f64 / rate;
            times.push(t);
            if t > max_time {
                max_time = t;
            }
        }

        for t in times {
            total_idle += max_time - t;
        }

        let opt = self.theoretical_optimal();
        SimulationResult {
            scheduler_type: "Fixed Segmentation".to_string(),
            total_file_bytes: self.file_size_bytes,
            worker_speeds_mbps: self.worker_speeds_bytes_per_sec.iter().map(|b| b / (1024.0 * 1024.0)).collect(),
            completion_time_seconds: max_time,
            theoretical_optimal_seconds: opt,
            efficiency_pct: (opt / max_time * 100.0).clamp(0.0, 100.0),
            total_splits: 0,
            worker_idle_time_seconds: total_idle,
        }
    }

    /// Simulation C: Largest-Segment Splitting (IDM-style 50/50 splits on largest active range).
    pub fn simulate_largest_segment(&self) -> SimulationResult {
        self.simulate_dynamic(false)
    }

    /// Simulation D: AURORA Expected Completion Time (ECT) with rate-proportional splitting.
    pub fn simulate_aurora_ect(&self) -> SimulationResult {
        self.simulate_dynamic(true)
    }

    fn simulate_dynamic(&self, use_ect_proportional: bool) -> SimulationResult {
        let num_workers = self.worker_speeds_bytes_per_sec.len();
        let mut worker_remaining = vec![self.file_size_bytes / num_workers as u64; num_workers];
        let mut sim_time = 0.0f64;
        let dt = 0.05f64; // 50ms step
        let mut total_splits = 0usize;
        let mut total_idle = 0.0f64;

        loop {
            let all_done = worker_remaining.iter().all(|&r| r == 0);
            if all_done {
                break;
            }

            sim_time += dt;

            // Advance download on all active workers
            for i in 0..num_workers {
                if worker_remaining[i] > 0 {
                    let bytes_transferred = (self.worker_speeds_bytes_per_sec[i] * dt) as u64;
                    worker_remaining[i] = worker_remaining[i].saturating_sub(bytes_transferred);
                } else {
                    total_idle += dt;
                }
            }

            // Check for idle workers that can steal work
            for idle_idx in 0..num_workers {
                if worker_remaining[idle_idx] == 0 {
                    if use_ect_proportional {
                        // Find worst straggler by Expected Completion Time: ECT = R / rate
                        let mut max_ect = 0.0;
                        let mut target_idx = None;

                        for active_idx in 0..num_workers {
                            if worker_remaining[active_idx] > self.min_segment_size * 2 {
                                let ect = worker_remaining[active_idx] as f64 / self.worker_speeds_bytes_per_sec[active_idx];
                                if ect > max_ect {
                                    max_ect = ect;
                                    target_idx = Some(active_idx);
                                }
                            }
                        }

                        if let Some(target) = target_idx {
                            let ra = self.worker_speeds_bytes_per_sec[target];
                            let rb = self.worker_speeds_bytes_per_sec[idle_idx];
                            let total_r = worker_remaining[target];

                            // Rate proportional allocation: Rb / (Ra + Rb)
                            let stolen = (total_r as f64 * (rb / (ra + rb))).round() as u64;
                            if stolen >= self.min_segment_size {
                                worker_remaining[target] -= stolen;
                                worker_remaining[idle_idx] = stolen;
                                total_splits += 1;
                            }
                        }
                    } else {
                        // Largest segment 50/50 split
                        let mut max_rem = 0;
                        let mut target_idx = None;

                        for active_idx in 0..num_workers {
                            if worker_remaining[active_idx] > max_rem && worker_remaining[active_idx] > self.min_segment_size * 2 {
                                max_rem = worker_remaining[active_idx];
                                target_idx = Some(active_idx);
                            }
                        }

                        if let Some(target) = target_idx {
                            let stolen = worker_remaining[target] / 2;
                            worker_remaining[target] -= stolen;
                            worker_remaining[idle_idx] = stolen;
                            total_splits += 1;
                        }
                    }
                }
            }
        }

        let opt = self.theoretical_optimal();
        let label = if use_ect_proportional {
            "AURORA-ECT (Rate-Proportional)"
        } else {
            "Largest-Segment Split (50/50)"
        };

        SimulationResult {
            scheduler_type: label.to_string(),
            total_file_bytes: self.file_size_bytes,
            worker_speeds_mbps: self.worker_speeds_bytes_per_sec.iter().map(|b| b / (1024.0 * 1024.0)).collect(),
            completion_time_seconds: sim_time,
            theoretical_optimal_seconds: opt,
            efficiency_pct: (opt / sim_time * 100.0).clamp(0.0, 100.0),
            total_splits,
            worker_idle_time_seconds: total_idle,
        }
    }
}
