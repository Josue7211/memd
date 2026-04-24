//! Per-event per-session advisory file lock.
//!
//! Contract: `docs/contracts/hook-order.md §6` — two concurrent fires of the
//! same event on the same session MUST serialize. Second waiter queues up
//! to `DEFAULT_WAIT_MS` (1 s) before erroring.
//!
//! Backed by `std::fs::File::try_lock` (stable since Rust 1.89). The lock
//! file lives at `{bundle}/state/session-{session_id}/hook.{event}.lock`
//! so each `(session, event)` pair gets an independent fcntl slot.

use super::HookEvent;
use std::fs::{File, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const POLL_INTERVAL: Duration = Duration::from_millis(10);
pub const DEFAULT_WAIT_MS: u64 = 1_000;

/// RAII guard: the kernel-level lock drops when this value drops.
#[derive(Debug)]
pub struct HookSessionLock {
    _file: File,
    path: PathBuf,
}

impl HookSessionLock {
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Try to acquire the `(session_id, event)` lock under `bundle_root`,
    /// polling every 10 ms until `wait` elapses.
    ///
    /// Returns `io::ErrorKind::WouldBlock` if another process still holds
    /// the lock after the deadline.
    pub fn acquire(
        bundle_root: &Path,
        session_id: &str,
        event: HookEvent,
        wait: Duration,
    ) -> io::Result<Self> {
        let path = lock_path(bundle_root, session_id, event);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .truncate(false)
            .open(&path)?;

        let deadline = Instant::now() + wait;
        loop {
            match file.try_lock() {
                Ok(()) => {
                    return Ok(HookSessionLock { _file: file, path });
                }
                Err(std::fs::TryLockError::WouldBlock) => {
                    if Instant::now() >= deadline {
                        return Err(io::Error::new(
                            io::ErrorKind::WouldBlock,
                            format!(
                                "hook lock contended: {} (waited {:?})",
                                path.display(),
                                wait
                            ),
                        ));
                    }
                    std::thread::sleep(POLL_INTERVAL);
                }
                Err(std::fs::TryLockError::Error(e)) => return Err(e),
            }
        }
    }
}

pub fn lock_path(bundle_root: &Path, session_id: &str, event: HookEvent) -> PathBuf {
    bundle_root
        .join("state")
        .join(format!("session-{session_id}"))
        .join(format!("hook.{}.lock", event.as_str()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Barrier};
    use std::thread;
    use tempfile::TempDir;

    #[test]
    fn single_acquire_releases_on_drop() {
        let dir = TempDir::new().unwrap();
        let guard = HookSessionLock::acquire(
            dir.path(),
            "sess-a",
            HookEvent::PreEdit,
            Duration::from_millis(100),
        )
        .unwrap();
        assert!(guard.path().exists());
        drop(guard);
        // Second acquire after drop should succeed immediately.
        let _g2 = HookSessionLock::acquire(
            dir.path(),
            "sess-a",
            HookEvent::PreEdit,
            Duration::from_millis(100),
        )
        .unwrap();
    }

    #[test]
    fn different_events_do_not_contend() {
        let dir = TempDir::new().unwrap();
        let _g1 = HookSessionLock::acquire(
            dir.path(),
            "sess-a",
            HookEvent::PreEdit,
            Duration::from_millis(100),
        )
        .unwrap();
        let _g2 = HookSessionLock::acquire(
            dir.path(),
            "sess-a",
            HookEvent::PreRead,
            Duration::from_millis(100),
        )
        .unwrap();
    }

    #[test]
    fn different_sessions_do_not_contend() {
        let dir = TempDir::new().unwrap();
        let _g1 = HookSessionLock::acquire(
            dir.path(),
            "sess-a",
            HookEvent::PreEdit,
            Duration::from_millis(100),
        )
        .unwrap();
        let _g2 = HookSessionLock::acquire(
            dir.path(),
            "sess-b",
            HookEvent::PreEdit,
            Duration::from_millis(100),
        )
        .unwrap();
    }

    #[test]
    fn contended_second_acquirer_errors_after_wait() {
        let dir = Arc::new(TempDir::new().unwrap());
        let barrier = Arc::new(Barrier::new(2));
        let dir_a = dir.clone();
        let bar_a = barrier.clone();

        let holder = thread::spawn(move || {
            let _g = HookSessionLock::acquire(
                dir_a.path(),
                "sess-c",
                HookEvent::PreEdit,
                Duration::from_millis(50),
            )
            .unwrap();
            bar_a.wait();
            thread::sleep(Duration::from_millis(200));
        });

        barrier.wait();
        let t0 = Instant::now();
        let err = HookSessionLock::acquire(
            dir.path(),
            "sess-c",
            HookEvent::PreEdit,
            Duration::from_millis(80),
        )
        .unwrap_err();
        let elapsed = t0.elapsed();
        assert_eq!(err.kind(), io::ErrorKind::WouldBlock);
        assert!(
            elapsed >= Duration::from_millis(60),
            "should wait ≥ 60ms, waited {elapsed:?}"
        );
        holder.join().unwrap();
    }
}
