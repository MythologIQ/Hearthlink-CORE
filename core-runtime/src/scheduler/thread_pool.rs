//! Configurable Thread Pool for optimized parallel inference.
//!
//! Provides work-stealing thread pool with configurable thread counts,
//! priority queues, and affinity settings for optimal CPU utilization.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex, RwLock};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

/// Configuration for the thread pool.
#[derive(Debug, Clone)]
pub struct ThreadPoolConfig {
    /// Number of worker threads (0 = auto-detect).
    pub num_threads: usize,
    /// Enable work stealing between threads.
    pub enable_work_stealing: bool,
    /// Queue size per worker.
    pub queue_size: usize,
    /// Thread stack size in bytes (0 = default).
    pub stack_size: usize,
    /// Thread name prefix.
    pub thread_name_prefix: String,
    /// Enable priority queue.
    pub enable_priority: bool,
    /// Idle timeout before thread sleeps (milliseconds).
    pub idle_timeout_ms: u64,
    /// Enable CPU affinity pinning.
    pub enable_affinity: bool,
}

impl Default for ThreadPoolConfig {
    fn default() -> Self {
        Self {
            num_threads: 0, // Auto-detect
            enable_work_stealing: true,
            queue_size: 256,
            stack_size: 0,
            thread_name_prefix: "core-worker".to_string(),
            enable_priority: true,
            idle_timeout_ms: 10,
            enable_affinity: false,
        }
    }
}

impl ThreadPoolConfig {
    /// Create config optimized for inference workloads.
    pub fn inference_optimized() -> Self {
        Self {
            num_threads: 0,
            enable_work_stealing: true,
            queue_size: 512,
            stack_size: 2 * 1024 * 1024, // 2MB
            thread_name_prefix: "inference".to_string(),
            enable_priority: true,
            idle_timeout_ms: 5,
            enable_affinity: true,
        }
    }

    /// Create config optimized for batch processing.
    pub fn batch_optimized() -> Self {
        Self {
            num_threads: 0,
            enable_work_stealing: true,
            queue_size: 1024,
            stack_size: 0,
            thread_name_prefix: "batch".to_string(),
            enable_priority: false,
            idle_timeout_ms: 50,
            enable_affinity: false,
        }
    }
}

/// Task priority levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// A task to be executed by the thread pool.
pub type Task = Box<dyn FnOnce() + Send + 'static>;

/// Prioritized task wrapper.
struct PrioritizedTask {
    task: Task,
    priority: TaskPriority,
    sequence: u64, // For FIFO ordering within same priority
}

/// Statistics for thread pool performance.
#[derive(Debug, Default, Clone)]
pub struct ThreadPoolStats {
    pub total_tasks_executed: u64,
    pub high_priority_tasks: u64,
    pub work_steals: u64,
    pub queue_overflows: u64,
    pub avg_wait_time_us: u64,
    pub avg_exec_time_us: u64,
    pub threads_active: usize,
    pub threads_idle: usize,
}

/// Worker thread state.
struct Worker {
    queue: Arc<Mutex<VecDeque<PrioritizedTask>>>,
    active: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

/// Configurable thread pool with work stealing.
pub struct ThreadPool {
    workers: Vec<Worker>,
    config: ThreadPoolConfig,
    stats: Arc<RwLock<ThreadPoolStats>>,
    task_sequence: AtomicU64,
    shutdown: Arc<AtomicBool>,
    condvar: Arc<(Mutex<bool>, Condvar)>,
    global_queue: Arc<Mutex<VecDeque<PrioritizedTask>>>,
    all_queues: Vec<Arc<Mutex<VecDeque<PrioritizedTask>>>>,
}

impl ThreadPool {
    /// Create a new thread pool with the given configuration.
    pub fn new(config: ThreadPoolConfig) -> Self {
        let num_threads = if config.num_threads == 0 {
            num_cpus::get().max(1)
        } else {
            config.num_threads
        };

        let shutdown = Arc::new(AtomicBool::new(false));
        let condvar = Arc::new((Mutex::new(false), Condvar::new()));
        let global_queue = Arc::new(Mutex::new(VecDeque::with_capacity(config.queue_size)));
        let stats = Arc::new(RwLock::new(ThreadPoolStats::default()));

        // Create worker queues
        let all_queues: Vec<Arc<Mutex<VecDeque<PrioritizedTask>>>> = (0..num_threads)
            .map(|_| Arc::new(Mutex::new(VecDeque::with_capacity(config.queue_size))))
            .collect();

        let mut workers = Vec::with_capacity(num_threads);

        for id in 0..num_threads {
            let queue = all_queues[id].clone();
            let queue_for_worker = queue.clone(); // Clone for worker struct

            // Create steal targets (all queues for stealing)
            let steal_queues = all_queues.clone();

            let active = Arc::new(AtomicBool::new(false));
            let shutdown_clone = shutdown.clone();
            let condvar_clone = condvar.clone();
            let global_queue_clone = global_queue.clone();
            let stats_clone = stats.clone();
            let config_clone = config.clone();
            let active_clone = active.clone();

            let thread_name = format!("{}-{}", config.thread_name_prefix, id);

            let handle = thread::Builder::new()
                .name(thread_name)
                .stack_size(if config.stack_size > 0 {
                    config.stack_size
                } else {
                    0
                })
                .spawn(move || {
                    Self::worker_loop(
                        id,
                        queue,
                        steal_queues,
                        active_clone,
                        shutdown_clone,
                        condvar_clone,
                        global_queue_clone,
                        stats_clone,
                        config_clone,
                    );
                })
                .expect("Failed to spawn worker thread");

            workers.push(Worker {
                queue: queue_for_worker,
                active,
                handle: Some(handle),
            });
        }

        Self {
            workers,
            config,
            stats,
            task_sequence: AtomicU64::new(0),
            shutdown,
            condvar,
            global_queue,
            all_queues,
        }
    }

    /// Submit a task with normal priority.
    pub fn submit(&self, task: Task) -> Result<(), ThreadPoolError> {
        self.submit_with_priority(task, TaskPriority::Normal)
    }

    /// Submit a task with specified priority.
    pub fn submit_with_priority(
        &self,
        task: Task,
        priority: TaskPriority,
    ) -> Result<(), ThreadPoolError> {
        if self.shutdown.load(Ordering::SeqCst) {
            return Err(ThreadPoolError::PoolShutdown);
        }

        let prioritized = PrioritizedTask {
            task,
            priority,
            sequence: self.task_sequence.fetch_add(1, Ordering::SeqCst),
        };

        // Find least loaded worker
        let min_queue_worker = self.find_least_loaded_worker();

        let queue = if let Some(id) = min_queue_worker {
            self.workers[id].queue.clone()
        } else {
            self.global_queue.clone()
        };

        {
            let mut q = queue.lock().unwrap();
            if q.len() >= self.config.queue_size {
                return Err(ThreadPoolError::QueueFull);
            }

            // Insert in priority order
            let insert_pos = q
                .iter()
                .position(|t| {
                    t.priority < prioritized.priority
                        || (t.priority == prioritized.priority && t.sequence > prioritized.sequence)
                })
                .unwrap_or(q.len());

            q.insert(insert_pos, prioritized);
        }

        // Wake up a worker
        let (lock, cvar) = &*self.condvar;
        {
            let _guard = lock.lock().unwrap();
            cvar.notify_one();
        }

        Ok(())
    }

    /// Find the worker with the smallest queue.
    fn find_least_loaded_worker(&self) -> Option<usize> {
        self.workers
            .iter()
            .enumerate()
            .min_by_key(|(_, w)| w.queue.lock().unwrap().len())
            .map(|(i, _)| i)
    }

    /// Worker thread main loop.
    fn worker_loop(
        worker_id: usize,
        queue: Arc<Mutex<VecDeque<PrioritizedTask>>>,
        all_queues: Vec<Arc<Mutex<VecDeque<PrioritizedTask>>>>,
        active: Arc<AtomicBool>,
        shutdown: Arc<AtomicBool>,
        condvar: Arc<(Mutex<bool>, Condvar)>,
        global_queue: Arc<Mutex<VecDeque<PrioritizedTask>>>,
        stats: Arc<RwLock<ThreadPoolStats>>,
        config: ThreadPoolConfig,
    ) {
        let idle_timeout = Duration::from_millis(config.idle_timeout_ms);

        while !shutdown.load(Ordering::SeqCst) {
            // Try to get a task from local queue
            let task = queue.lock().unwrap().pop_front();

            let task = match task {
                Some(t) => Some(t),
                None => {
                    // Try global queue
                    if let Some(t) = global_queue.lock().unwrap().pop_front() {
                        Some(t)
                    } else if config.enable_work_stealing {
                        // Try to steal from other workers
                        Self::try_steal(worker_id, &all_queues)
                    } else {
                        None
                    }
                }
            };

            if let Some(prioritized) = task {
                active.store(true, Ordering::SeqCst);

                let start = Instant::now();
                (prioritized.task)();
                let exec_time = start.elapsed();

                // Update stats
                if let Ok(mut s) = stats.write() {
                    s.total_tasks_executed += 1;
                    if prioritized.priority >= TaskPriority::High {
                        s.high_priority_tasks += 1;
                    }
                    // Rolling average of execution time
                    let exec_us = exec_time.as_micros() as u64;
                    if s.avg_exec_time_us == 0 {
                        s.avg_exec_time_us = exec_us;
                    } else {
                        s.avg_exec_time_us = (s.avg_exec_time_us * 9 + exec_us) / 10;
                    }
                }

                active.store(false, Ordering::SeqCst);
            } else {
                // No work available, wait
                let (lock, cvar) = &*condvar;
                let _guard = cvar
                    .wait_timeout(lock.lock().unwrap(), idle_timeout)
                    .unwrap();
            }
        }
    }

    /// Try to steal a task from other workers.
    fn try_steal(
        worker_id: usize,
        all_queues: &[Arc<Mutex<VecDeque<PrioritizedTask>>>],
    ) -> Option<PrioritizedTask> {
        for (id, target) in all_queues.iter().enumerate() {
            if id == worker_id {
                continue; // Don't steal from self
            }
            let mut q = target.lock().unwrap();
            if let Some(task) = q.pop_back() {
                return Some(task);
            }
        }
        None
    }

    /// Get current statistics.
    pub fn stats(&self) -> ThreadPoolStats {
        let mut stats = self.stats.read().unwrap().clone();
        stats.threads_active = self
            .workers
            .iter()
            .filter(|w| w.active.load(Ordering::SeqCst))
            .count();
        stats.threads_idle = self.workers.len() - stats.threads_active;
        stats
    }

    /// Get number of worker threads.
    pub fn num_threads(&self) -> usize {
        self.workers.len()
    }

    /// Check if pool is shutting down.
    pub fn is_shutdown(&self) -> bool {
        self.shutdown.load(Ordering::SeqCst)
    }

    /// Signal shutdown (does not wait for threads).
    pub fn signal_shutdown(&self) {
        self.shutdown.store(true, Ordering::SeqCst);
        let (lock, cvar) = &*self.condvar;
        {
            let _guard = lock.lock().unwrap();
            cvar.notify_all();
        }
    }

    /// Wait for all workers to finish and consume the pool.
    pub fn join(mut self) {
        self.shutdown.store(true, Ordering::SeqCst);

        // Wake all workers
        let (lock, cvar) = &*self.condvar;
        {
            let _guard = lock.lock().unwrap();
            cvar.notify_all();
        }

        // Wait for all workers to finish
        for worker in self.workers.drain(..) {
            if let Some(handle) = worker.handle {
                let _ = handle.join();
            }
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::SeqCst);
        let (lock, cvar) = &*self.condvar;
        {
            let _guard = lock.lock().unwrap();
            cvar.notify_all();
        }

        // Try to join threads on drop
        for worker in self.workers.drain(..) {
            if let Some(handle) = worker.handle {
                let _ = handle.join();
            }
        }
    }
}

/// Errors for thread pool operations.
#[derive(Debug, thiserror::Error)]
pub enum ThreadPoolError {
    #[error("Thread pool is shut down")]
    PoolShutdown,

    #[error("Task queue is full")]
    QueueFull,

    #[error("Failed to spawn thread: {0}")]
    ThreadSpawnFailed(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;

    #[test]
    fn test_thread_pool_basic() {
        let pool = ThreadPool::new(ThreadPoolConfig::default());
        assert!(pool.num_threads() > 0);

        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        pool.submit(Box::new(move || {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        }))
        .unwrap();

        // Wait for task to complete
        thread::sleep(Duration::from_millis(100));

        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_priority_tasks() {
        let config = ThreadPoolConfig {
            enable_priority: true,
            ..Default::default()
        };
        let pool = ThreadPool::new(config);

        let results = Arc::new(Mutex::new(Vec::new()));

        // Submit tasks with different priorities
        for i in 0..3 {
            let results_clone = results.clone();
            pool.submit_with_priority(
                Box::new(move || {
                    results_clone.lock().unwrap().push(format!("normal-{}", i));
                }),
                TaskPriority::Normal,
            )
            .unwrap();
        }

        pool.submit_with_priority(
            Box::new(|| {
                // High priority task
            }),
            TaskPriority::High,
        )
        .unwrap();

        thread::sleep(Duration::from_millis(100));

        let stats = pool.stats();
        assert!(stats.total_tasks_executed >= 3);
    }

    #[test]
    fn test_config_presets() {
        let inference_config = ThreadPoolConfig::inference_optimized();
        assert!(inference_config.enable_work_stealing);
        assert!(inference_config.enable_priority);

        let batch_config = ThreadPoolConfig::batch_optimized();
        assert!(batch_config.enable_work_stealing);
        assert!(!batch_config.enable_priority);
    }

    #[test]
    fn test_stats_tracking() {
        let pool = ThreadPool::new(ThreadPoolConfig::default());

        for _ in 0..10 {
            pool.submit(Box::new(|| {})).unwrap();
        }

        thread::sleep(Duration::from_millis(200));

        let stats = pool.stats();
        assert!(stats.total_tasks_executed >= 10);
    }
}
