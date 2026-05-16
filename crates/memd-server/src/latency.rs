// K2.6: Process-global latency histogram for working-memory retrieval.
// Log-base-2 bucketed, atomic counters — cheap enough to sit on every
// request without touching a mutex. `percentile()` is a linear scan over
// `BUCKET_COUNT` cells and returns the upper edge of the bucket the
// target rank falls into, so it's intentionally an overestimate.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use memd_schema::{LatencyBucket, LatencyDiagnosticsResponse};

pub(crate) const BUCKET_COUNT: usize = 22;
const RECENT_SAMPLE_LIMIT: usize = 128;

pub(crate) struct LatencyHistogram {
    buckets: [AtomicU64; BUCKET_COUNT],
    total: AtomicU64,
    max_ms: AtomicU64,
    sum_ms: AtomicU64,
    recent: Mutex<VecDeque<u64>>,
}

impl LatencyHistogram {
    pub(crate) fn new() -> Arc<Self> {
        Arc::new(Self {
            buckets: std::array::from_fn(|_| AtomicU64::new(0)),
            total: AtomicU64::new(0),
            max_ms: AtomicU64::new(0),
            sum_ms: AtomicU64::new(0),
            recent: Mutex::new(VecDeque::with_capacity(RECENT_SAMPLE_LIMIT)),
        })
    }

    pub(crate) fn record_ms(&self, ms: u64) {
        let idx = bucket_for_ms(ms);
        self.buckets[idx].fetch_add(1, Ordering::Relaxed);
        self.total.fetch_add(1, Ordering::Relaxed);
        self.sum_ms.fetch_add(ms, Ordering::Relaxed);
        self.max_ms.fetch_max(ms, Ordering::Relaxed);
        if let Ok(mut recent) = self.recent.lock() {
            if recent.len() == RECENT_SAMPLE_LIMIT {
                recent.pop_front();
            }
            recent.push_back(ms);
        }
    }

    pub(crate) fn snapshot(&self) -> LatencyDiagnosticsResponse {
        let buckets: Vec<u64> = self
            .buckets
            .iter()
            .map(|b| b.load(Ordering::Relaxed))
            .collect();
        let total: u64 = self.total.load(Ordering::Relaxed);
        let max_ms = self.max_ms.load(Ordering::Relaxed);
        let sum_ms = self.sum_ms.load(Ordering::Relaxed);
        let mean_ms = if total == 0 {
            0.0
        } else {
            sum_ms as f64 / total as f64
        };
        let recent_samples = self.recent_samples();
        let recent_total = recent_samples.len() as u64;
        let recent_p95_ms = percentile_for_samples(&recent_samples, 0.95);

        let bucket_records: Vec<LatencyBucket> = buckets
            .iter()
            .enumerate()
            .map(|(i, count)| LatencyBucket {
                upper_ms: bucket_upper_ms(i),
                count: *count,
            })
            .collect();

        LatencyDiagnosticsResponse {
            surface: "working_memory".to_string(),
            total,
            recent_total,
            mean_ms,
            max_ms,
            p50_ms: percentile(&buckets, total, 0.50),
            p95_ms: percentile(&buckets, total, 0.95),
            p99_ms: percentile(&buckets, total, 0.99),
            recent_p95_ms,
            buckets: bucket_records,
        }
    }

    pub(crate) fn recent_p95_ms(&self) -> Option<f64> {
        let samples = self.recent_samples();
        if samples.is_empty() {
            None
        } else {
            Some(percentile_for_samples(&samples, 0.95))
        }
    }

    fn recent_samples(&self) -> Vec<u64> {
        self.recent
            .lock()
            .map(|recent| recent.iter().copied().collect())
            .unwrap_or_default()
    }
}

// Bucket i covers [2^i ms, 2^(i+1) ms). Bucket 0 also absorbs submillisecond
// samples (we record whole-ms durations, so <1ms clamps to bucket 0).
fn bucket_for_ms(ms: u64) -> usize {
    if ms == 0 {
        return 0;
    }
    let ilog = 64 - ms.leading_zeros() as usize - 1;
    ilog.min(BUCKET_COUNT - 1)
}

fn bucket_upper_ms(i: usize) -> u64 {
    1u64 << (i + 1).min(63)
}

fn percentile(buckets: &[u64], total: u64, q: f64) -> f64 {
    if total == 0 {
        return 0.0;
    }
    let target = ((total as f64) * q).ceil() as u64;
    let mut cumulative: u64 = 0;
    for (i, count) in buckets.iter().enumerate() {
        cumulative += *count;
        if cumulative >= target {
            return bucket_upper_ms(i) as f64;
        }
    }
    bucket_upper_ms(BUCKET_COUNT - 1) as f64
}

fn percentile_for_samples(samples: &[u64], q: f64) -> f64 {
    if samples.is_empty() {
        return 0.0;
    }
    let mut buckets = vec![0u64; BUCKET_COUNT];
    for sample in samples {
        buckets[bucket_for_ms(*sample)] += 1;
    }
    percentile(&buckets, samples.len() as u64, q)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bucket_boundaries() {
        assert_eq!(bucket_for_ms(0), 0);
        assert_eq!(bucket_for_ms(1), 0);
        assert_eq!(bucket_for_ms(2), 1);
        assert_eq!(bucket_for_ms(3), 1);
        assert_eq!(bucket_for_ms(4), 2);
        assert_eq!(bucket_for_ms(100), 6); // 64..128 bucket
    }

    #[test]
    fn records_and_summarises() {
        let hist = LatencyHistogram::new();
        for _ in 0..95 {
            hist.record_ms(5);
        }
        for _ in 0..5 {
            hist.record_ms(200);
        }
        let snap = hist.snapshot();
        assert_eq!(snap.total, 100);
        assert!(snap.p50_ms <= 16.0, "p50 should fall in small bucket");
        assert!(
            snap.p95_ms <= 16.0 || snap.p95_ms >= 128.0,
            "p95 sits at bucket boundary"
        );
        assert!(snap.p99_ms >= 128.0);
    }

    #[test]
    fn recent_p95_drops_stale_outliers() {
        let hist = LatencyHistogram::new();
        for _ in 0..100 {
            hist.record_ms(1500);
        }
        for _ in 0..RECENT_SAMPLE_LIMIT {
            hist.record_ms(5);
        }

        let snap = hist.snapshot();

        assert_eq!(snap.total, 228);
        assert_eq!(snap.recent_total, RECENT_SAMPLE_LIMIT as u64);
        assert!(snap.p95_ms >= 1024.0, "all-time p95 keeps old outliers");
        assert!(
            snap.recent_p95_ms <= 16.0,
            "recent p95 should track current tail, got {}",
            snap.recent_p95_ms
        );
        assert_eq!(hist.recent_p95_ms(), Some(snap.recent_p95_ms));
    }
}
