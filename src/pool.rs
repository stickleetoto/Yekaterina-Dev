//! In-process worker pool for pure compute jobs.
//!
//! The only module permitted to create OS threads; `scripts/static_audit_v11.py`
//! fails the build if thread creation appears anywhere else. Ad-hoc threading is
//! how ordering and determinism bugs enter a codebase that currently has none.
//!
//! # Why OS threads rather than tokio tasks
//!
//! `rmcp` already dispatches every `tools/call` as a tokio task, and those tasks
//! run on the runtime's worker threads. Compute here is CPU-bound and
//! unyielding: `signal.dft` over 2,048 samples took 47 ms on the frozen
//! baseline, and a legal `ode.rk4` request may run for far longer. Executing
//! that on a runtime worker blocks the I/O driver and the response sink. Real
//! threads keep compute off the runtime entirely.
//!
//! # Data flow
//!
//! [`Job`] in, [`JobResult`] out, both plain owned data with no borrow of server
//! state. That is deliberate: it is the seam a future process or remote backend
//! would attach to, without the scheduler's ordering and dependency logic
//! needing to change. No trait hierarchy is introduced for that today -- there
//! is exactly one implementation, and the audit forbids spawning a subprocess,
//! so a second one cannot exist yet.
//!
//! # Panics
//!
//! A panicking job must not take down a worker, and must not change what a
//! client observes. Jobs run under `catch_unwind`; the payload is carried back
//! and resumed on the request task by [`WorkerPool::run`].
//!
//! That reproduces v1.0.0 exactly. There, a panic inside `engine::execute`
//! unwound through the request's tokio task: the task died, no response was
//! sent, and the server survived. Here the same panic reaches the same place
//! with the same payload; only the worker thread is spared. No new error code is
//! introduced, and no externally observable behaviour changes.

use std::any::Any;
use std::collections::VecDeque;
use std::panic::{self, AssertUnwindSafe};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::JoinHandle;

use serde_json::Value;

use crate::engine;

/// Upper bound on `--workers auto`.
///
/// Deliberately conservative for a first parallel release. The batch limit is
/// 1,024 items and each in-flight job may hold a large intermediate `Value`, so
/// worker count is also a memory multiplier. Raise it only with the 1/2/4/8
/// scaling data from Phase 9 in hand.
pub const MAX_AUTO_WORKERS: usize = 8;

/// One unit of pure work.
///
/// `opcode` is the canonical opcode, already resolved and classified
/// [`crate::safety::Safety::Pure`] by the caller. Owned rather than borrowed
/// because it crosses a thread boundary.
#[derive(Debug, Clone)]
pub struct Job {
    /// Position in the caller's input sequence. Results are placed by this
    /// index, never by completion order.
    pub index: usize,
    pub opcode: String,
    pub args: Vec<Value>,
}

/// What happened to one job.
pub enum Outcome {
    /// The operation ran and produced a value or an error code.
    Done(Result<Value, &'static str>),
    /// The operation panicked. The payload is resumed on the request task.
    Panicked(Box<dyn Any + Send>),
}

pub struct JobResult {
    pub index: usize,
    pub outcome: Outcome,
}

/// Run one job. Pure by construction: takes no state, returns owned data.
fn run_job(job: &Job) -> Outcome {
    match panic::catch_unwind(AssertUnwindSafe(|| engine::execute(&job.opcode, &job.args))) {
        Ok(result) => Outcome::Done(result),
        Err(payload) => Outcome::Panicked(payload),
    }
}

struct Task {
    job: Job,
    results: Sender<JobResult>,
}

#[derive(Default)]
struct Queue {
    tasks: VecDeque<Task>,
    shutdown: bool,
}

struct Shared {
    queue: Mutex<Queue>,
    ready: Condvar,
}

/// A fixed set of worker threads sharing one job queue.
///
/// Threads are created once, at construction, and parked on a condition
/// variable until work arrives. Nothing is spawned per request.
pub struct WorkerPool {
    shared: Arc<Shared>,
    handles: Vec<JoinHandle<()>>,
    workers: usize,
}

impl WorkerPool {
    /// Create a pool with `workers` threads, clamped to at least one.
    pub fn new(workers: usize) -> Self {
        let workers = workers.max(1);
        let shared = Arc::new(Shared {
            queue: Mutex::new(Queue::default()),
            ready: Condvar::new(),
        });
        let mut handles = Vec::with_capacity(workers);
        for i in 0..workers {
            let shared = Arc::clone(&shared);
            let handle = std::thread::Builder::new()
                .name(format!("yk-worker-{i}"))
                .spawn(move || worker_loop(&shared))
                .expect("spawn worker thread");
            handles.push(handle);
        }
        Self { shared, handles, workers }
    }

    pub fn workers(&self) -> usize {
        self.workers
    }

    /// Execute `jobs` and return their results.
    ///
    /// Results come back in **input index order**, reconstructed by slot, never
    /// by completion order. If any job panicked, its payload is resumed here, on
    /// the calling thread, after every other job has been collected.
    ///
    /// Blocking: intended to be called from `spawn_blocking` or a non-async
    /// context, never directly on a tokio runtime worker.
    pub fn run(&self, jobs: Vec<Job>) -> Vec<Result<Value, &'static str>> {
        let n = jobs.len();
        if n == 0 {
            return Vec::new();
        }
        let (tx, rx): (Sender<JobResult>, Receiver<JobResult>) = mpsc::channel();
        {
            let mut queue = self.shared.queue.lock().expect("worker queue poisoned");
            for job in jobs {
                queue.tasks.push_back(Task { job, results: tx.clone() });
            }
        }
        // Wake exactly as many workers as there is work for.
        if n == 1 {
            self.shared.ready.notify_one();
        } else {
            self.shared.ready.notify_all();
        }
        drop(tx);

        let mut slots: Vec<Option<Result<Value, &'static str>>> = (0..n).map(|_| None).collect();
        let mut panic_payload: Option<Box<dyn Any + Send>> = None;
        for result in rx.iter().take(n) {
            match result.outcome {
                Outcome::Done(value) => slots[result.index] = Some(value),
                Outcome::Panicked(payload) => {
                    // Keep draining so no worker is left blocked on send, then
                    // resume the first panic once collection is complete.
                    slots[result.index] = Some(Err("NYI"));
                    if panic_payload.is_none() {
                        panic_payload = Some(payload);
                    }
                }
            }
        }
        if let Some(payload) = panic_payload {
            panic::resume_unwind(payload);
        }
        slots
            .into_iter()
            .map(|slot| slot.expect("every job index filled exactly once"))
            .collect()
    }
}

impl Drop for WorkerPool {
    fn drop(&mut self) {
        {
            let mut queue = self.shared.queue.lock().expect("worker queue poisoned");
            queue.shutdown = true;
        }
        self.shared.ready.notify_all();
        for handle in self.handles.drain(..) {
            let _ = handle.join();
        }
    }
}

fn worker_loop(shared: &Shared) {
    loop {
        let task = {
            let mut queue = shared.queue.lock().expect("worker queue poisoned");
            loop {
                if let Some(task) = queue.tasks.pop_front() {
                    break task;
                }
                if queue.shutdown {
                    return;
                }
                queue = shared.ready.wait(queue).expect("worker queue poisoned");
            }
        };
        let outcome = run_job(&task.job);
        // A closed receiver means the requester gave up; nothing to report.
        let _ = task.results.send(JobResult { index: task.job.index, outcome });
    }
}

/// Resolve a `--workers` setting.
///
/// `"auto"` uses the platform's available parallelism, clamped to
/// [`MAX_AUTO_WORKERS`]. `available_parallelism` respects cgroup and Windows
/// affinity limits, which a raw CPU count does not.
pub fn resolve_workers(setting: &str) -> Result<usize, String> {
    let setting = setting.trim();
    if setting.eq_ignore_ascii_case("auto") {
        let n = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1);
        return Ok(n.clamp(1, MAX_AUTO_WORKERS));
    }
    match setting.parse::<usize>() {
        Ok(0) => Err("--workers must be at least 1".to_string()),
        Ok(n) => Ok(n),
        Err(_) => Err(format!("--workers expects a positive integer or 'auto', got {setting:?}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn add(index: usize, a: i64, b: i64) -> Job {
        Job { index, opcode: "math.add".to_string(), args: vec![json!(a), json!(b)] }
    }

    #[test]
    fn single_worker_is_a_first_class_path() {
        let pool = WorkerPool::new(1);
        assert_eq!(pool.workers(), 1);
        let out = pool.run((0..16).map(|i| add(i, i as i64, 1)).collect());
        let want: Vec<_> = (0..16).map(|i| Ok(json!(i as f64 + 1.0))).collect();
        assert_eq!(out, want);
    }

    #[test]
    fn zero_workers_is_clamped_to_one() {
        assert_eq!(WorkerPool::new(0).workers(), 1);
    }

    #[test]
    fn results_are_in_input_order_regardless_of_worker_count() {
        for workers in [1, 2, 4, 8] {
            let pool = WorkerPool::new(workers);
            let out = pool.run((0..64).map(|i| add(i, i as i64 * 3, 7)).collect());
            let want: Vec<_> = (0..64).map(|i| Ok(json!(i as f64 * 3.0 + 7.0))).collect();
            assert_eq!(out, want, "workers={workers}");
        }
    }

    /// Skewed durations guarantee completion order differs from input order, so
    /// this fails immediately if results were collected by completion.
    #[test]
    fn skewed_durations_do_not_disturb_result_order() {
        let pool = WorkerPool::new(4);
        let signal: Vec<f64> = (0..256).map(|i| (i % 13) as f64).collect();
        let jobs: Vec<Job> = (0..16)
            .map(|i| {
                if i % 2 == 0 {
                    // Slow: naive O(n^2) transform.
                    Job { index: i, opcode: "signal.dft".to_string(), args: vec![json!(signal)] }
                } else {
                    add(i, i as i64, 0)
                }
            })
            .collect();
        let out = pool.run(jobs);
        assert_eq!(out.len(), 16);
        for (i, r) in out.iter().enumerate() {
            if i % 2 == 1 {
                assert_eq!(r.as_ref().unwrap(), &json!(i as f64), "slot {i} holds the wrong job");
            } else {
                assert!(r.is_ok(), "slot {i}");
            }
        }
    }

    #[test]
    fn error_codes_pass_through_unchanged() {
        let pool = WorkerPool::new(2);
        let jobs = vec![
            Job { index: 0, opcode: "math.div".into(), args: vec![json!(1), json!(0)] },
            Job { index: 1, opcode: "math.add".into(), args: vec![] },
            Job { index: 2, opcode: "zzz.nope".into(), args: vec![] },
            add(3, 2, 2),
        ];
        let out = pool.run(jobs);
        assert_eq!(out[0], Err("DIV0"));
        assert_eq!(out[1], Err("ARG"));
        assert_eq!(out[2], Err("OP"));
        assert_eq!(out[3], Ok(json!(4.0)));
    }

    #[test]
    fn empty_batch_is_a_no_op() {
        let pool = WorkerPool::new(4);
        assert!(pool.run(Vec::new()).is_empty());
    }

    #[test]
    fn pool_survives_many_batches_without_growing_threads() {
        let pool = WorkerPool::new(4);
        for round in 0..200 {
            let out = pool.run((0..8).map(|i| add(i, round, i as i64)).collect());
            assert_eq!(out.len(), 8);
            assert_eq!(out[3], Ok(json!(round as f64 + 3.0)));
        }
    }

    #[test]
    fn resolve_workers_parses_and_clamps() {
        assert_eq!(resolve_workers("1").unwrap(), 1);
        assert_eq!(resolve_workers("4").unwrap(), 4);
        assert_eq!(resolve_workers(" 8 ").unwrap(), 8);
        let auto = resolve_workers("auto").unwrap();
        assert!((1..=MAX_AUTO_WORKERS).contains(&auto), "auto={auto}");
        assert_eq!(resolve_workers("AUTO").unwrap(), auto);
        assert!(resolve_workers("0").is_err());
        assert!(resolve_workers("-1").is_err());
        assert!(resolve_workers("many").is_err());
    }
}
