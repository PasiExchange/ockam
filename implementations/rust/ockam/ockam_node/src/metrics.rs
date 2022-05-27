use crate::tokio::{runtime::Runtime, time};
use core::{
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};
use ockam_core::compat::{collections::BTreeMap, sync::Arc};
use std::{fs::OpenOptions, io::Write};

pub struct Metrics {
    rt: Arc<Runtime>,
}

impl Metrics {
    /// Create a new Metrics collector with access to the runtime
    pub(crate) fn new(rt: &Arc<Runtime>) -> Arc<Self> {
        Arc::new(Self { rt: Arc::clone(rt) })
    }

    /// Spawned by the Executor to periodically collect metrics
    pub(crate) async fn run(self: Arc<Self>, alive: Arc<AtomicBool>) {
        let path = match std::env::var("OCKAM_METRICS_PATH") {
            Ok(path) => path,
            Err(_) => {
                debug!("Metrics collection disabled, set `OCKAM_METRICS_PATH` to collect metrics");
                return;
            }
        };

        let mut file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(path)
            .expect("failed to open or create metrics collection file");

        file.write_all(b"Worker busy time (% since last poll)\n")
            .expect("failed to write metrics");

        let freq_ms = 100;
        let mut acc = MetricsReport::default();

        loop {
            if !alive.load(Ordering::Relaxed) {
                debug!("Metrics collector shutting down...");
                break;
            }

            let report = self.generate_report(freq_ms, &mut acc);

            file.write_all(format!("{}\n", report.to_csv()).as_bytes())
                .expect("failed to write metrics");
            time::sleep(Duration::from_millis(freq_ms)).await;
        }
    }

    pub(crate) fn generate_report(
        self: &Arc<Self>,
        freq: u64,
        acc: &mut MetricsReport,
    ) -> MetricsReport {
        let m = self.rt.metrics();

        let tokio_workers = m.num_workers();
        // let io_ready_count = m.io_driver_ready_count();

        let mut worker_busy_ms = BTreeMap::new();
        for wid in 0..tokio_workers {
            // Get the previously accumulated
            let acc_ms = acc.worker_busy_ms.get(&wid).unwrap_or(&0);
            let raw_ms = m.worker_total_busy_duration(wid).as_millis();

            let diff_ms = raw_ms - acc_ms;
            let percent = diff_ms as f32 / freq as f32;

            worker_busy_ms.insert(wid, percent as u128);
            acc.worker_busy_ms.insert(wid, raw_ms);
        }

        MetricsReport { worker_busy_ms }
    }
}

#[derive(Default)]
pub struct MetricsReport {
    worker_busy_ms: BTreeMap<usize, u128>,
}

impl MetricsReport {
    /// Generate a line of CSV for this report
    pub fn to_csv(&self) -> String {
        format!(
            "{}",
            self.worker_busy_ms
                .iter()
                .map(|(wid, depth)| format!("({}:{}%)", wid, depth))
                .collect::<Vec<String>>()
                .join(",")
        )
    }
}
