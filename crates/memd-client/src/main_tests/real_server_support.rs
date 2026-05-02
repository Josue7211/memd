//! Subprocess fixture for spawning a real `memd-server` binary in
//! ignored / nightly tests. The server crate is binary-only (no
//! library surface), so real-backend integration tests must launch a
//! child process. This module wraps spawn + readiness probe + cleanup
//! so individual tests stay focused on the assertion under test.
//!
//! Use `MEMD_SERVER_BIN` to override binary lookup (CI sets this to
//! the artifact path); otherwise the helper walks from
//! `CARGO_MANIFEST_DIR` to the workspace target dir and looks for
//! `debug/memd-server` then `release/memd-server`.

use anyhow::{Context, Result, anyhow};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

/// Handle to a spawned `memd-server` process. Drop kills the child
/// and waits on it; the tempdir housing the DB drops alongside.
pub(crate) struct SpawnedServer {
    child: Child,
    pub(crate) base_url: String,
    pub(crate) bundle_root: PathBuf,
    _tempdir: tempfile::TempDir,
}

impl Drop for SpawnedServer {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

pub(crate) fn locate_memd_server_bin() -> Result<PathBuf> {
    if let Ok(p) = std::env::var("MEMD_SERVER_BIN") {
        let p = PathBuf::from(p);
        if p.is_file() {
            return Ok(p);
        }
    }
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace = manifest
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| anyhow!("cannot derive workspace root from {manifest:?}"))?;
    let target_dir = std::env::var("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| workspace.join("target"));
    for profile in ["debug", "release"] {
        let candidate = target_dir.join(profile).join("memd-server");
        if candidate.is_file() {
            return Ok(candidate);
        }
    }
    Err(anyhow!(
        "memd-server binary not found under {target_dir:?}; run `cargo build --bin memd-server` first"
    ))
}

/// Spawn `memd-server` against a fresh tempdir DB on an ephemeral port.
/// Pre-binds `127.0.0.1:0` to capture a free port, drops the listener,
/// and passes the port to the subprocess via `MEMD_BIND_ADDR`. Polls
/// `/healthz` until it returns 2xx (max 10 s).
pub(crate) fn spawn_memd_server() -> Result<SpawnedServer> {
    let bin = locate_memd_server_bin()?;
    let tempdir = tempfile::tempdir().context("tempdir for memd-server")?;
    let bundle_root = tempdir.path().join(".memd");
    std::fs::create_dir_all(&bundle_root).context("create bundle root")?;
    let db_path = bundle_root.join("memd.db");

    let listener = TcpListener::bind("127.0.0.1:0").context("pre-bind ephemeral port")?;
    let port = listener.local_addr().context("local_addr")?.port();
    drop(listener);
    let bind_addr = format!("127.0.0.1:{port}");
    let base_url = format!("http://{bind_addr}");

    let child = Command::new(&bin)
        .env("MEMD_BIND_ADDR", &bind_addr)
        .env("MEMD_DB_PATH", db_path.to_string_lossy().to_string())
        .env("MEMD_LOG_FORMAT", "compact")
        .env("MEMD_RATE_LIMIT_DISABLED", "1")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .with_context(|| format!("spawn {bin:?}"))?;

    let deadline = Instant::now() + Duration::from_secs(10);
    let probe_url = format!("{base_url}/healthz");
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_millis(500))
        .build()
        .context("build probe client")?;
    let mut last_err: Option<String> = None;
    while Instant::now() < deadline {
        match client.get(&probe_url).send() {
            Ok(resp) if resp.status().is_success() => {
                return Ok(SpawnedServer {
                    child,
                    base_url,
                    bundle_root,
                    _tempdir: tempdir,
                });
            }
            Ok(resp) => last_err = Some(format!("status {}", resp.status())),
            Err(e) => last_err = Some(format!("{e}")),
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    Err(anyhow!(
        "memd-server failed to become healthy in 10s at {probe_url}: {last_err:?}"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Smoke test for the spawn helper. Ignored by default: requires a
    /// pre-built `memd-server` binary. Run with:
    ///   cargo test -p memd-client --bin memd -- --ignored real_server
    #[test]
    #[ignore]
    fn real_server_spawns_and_responds_to_healthz() {
        let server = spawn_memd_server().expect("spawn memd-server");
        let resp =
            reqwest::blocking::get(format!("{}/healthz", server.base_url)).expect("GET /healthz");
        assert!(
            resp.status().is_success(),
            "/healthz returned {}",
            resp.status()
        );
    }
}
