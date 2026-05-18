use super::*;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::net::{TcpStream, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct DevServerCommandResponse {
    pub(crate) action: String,
    pub(crate) url: String,
    pub(crate) scope: String,
    pub(crate) repo_root: String,
    pub(crate) repo_hash: String,
    pub(crate) leases: Vec<memd_schema::DevServerLeaseRecord>,
    pub(crate) exit_code: Option<i32>,
    pub(crate) summary_mode: bool,
}

pub(crate) async fn run_dev_server_command(
    args: &DevServerArgs,
    base_url: &str,
) -> anyhow::Result<DevServerCommandResponse> {
    match &args.command {
        DevServerSubcommand::Guard(args) => run_dev_server_guard(args, base_url).await,
        DevServerSubcommand::List(args) => run_dev_server_list(args, base_url).await,
        DevServerSubcommand::Release(args) => run_dev_server_release(args, base_url).await,
    }
}

async fn run_dev_server_guard(
    args: &DevServerGuardArgs,
    base_url: &str,
) -> anyhow::Result<DevServerCommandResponse> {
    let context = build_dev_server_context(&args.output, base_url, &args.host, args.port)?;
    if port_open(&args.host, args.port) {
        return Ok(DevServerCommandResponse {
            action: "existing".to_string(),
            url: context.url,
            scope: context.scope,
            repo_root: context.repo_root,
            repo_hash: context.repo_hash,
            leases: Vec::new(),
            exit_code: None,
            summary_mode: args.summary,
        });
    }
    if args.command.is_empty() {
        anyhow::bail!("dev-server guard requires a command when the port is free");
    }
    refuse_cross_project_clawcontrol_dev_server(&context.repo_root, &args.command)?;

    let client = MemdClient::new(&context.base_url)?;
    let acquire =
        context.acquire_request(&args.command, None, args.ttl_secs, args.stale_after_secs);
    client.acquire_dev_server_lease(&acquire).await?;

    if port_open(&args.host, args.port) {
        client
            .release_dev_server_lease(&context.release_request())
            .await?;
        return Ok(DevServerCommandResponse {
            action: "existing_after_claim".to_string(),
            url: context.url,
            scope: context.scope,
            repo_root: context.repo_root,
            repo_hash: context.repo_hash,
            leases: Vec::new(),
            exit_code: None,
            summary_mode: args.summary,
        });
    }

    let mut child = Command::new(&args.command[0])
        .args(&args.command[1..])
        .spawn()
        .with_context(|| format!("spawn dev server command: {}", args.command.join(" ")))?;
    let pid = child.id();
    let leases = client
        .acquire_dev_server_lease(&context.acquire_request(
            &args.command,
            Some(pid),
            args.ttl_secs,
            args.stale_after_secs,
        ))
        .await?
        .leases;

    let heartbeat_done = Arc::new(AtomicBool::new(false));
    let heartbeat_done_task = Arc::clone(&heartbeat_done);
    let heartbeat_client = client.clone();
    let heartbeat_req = context.acquire_request(
        &args.command,
        Some(pid),
        args.ttl_secs,
        args.stale_after_secs,
    );
    let heartbeat_interval = Duration::from_secs(args.ttl_secs.clamp(3, 90) / 3);
    let heartbeat = tokio::spawn(async move {
        while !heartbeat_done_task.load(Ordering::Relaxed) {
            tokio::time::sleep(heartbeat_interval).await;
            if heartbeat_done_task.load(Ordering::Relaxed) {
                break;
            }
            let _ = heartbeat_client
                .acquire_dev_server_lease(&heartbeat_req)
                .await;
        }
    });

    let status = tokio::task::spawn_blocking(move || child.wait())
        .await
        .context("join dev server command wait")?
        .context("wait dev server command")?;
    heartbeat_done.store(true, Ordering::Relaxed);
    heartbeat.abort();
    let _ = client
        .release_dev_server_lease(&context.release_request())
        .await;

    if !status.success() {
        return Err(exit_status_error(status, &args.command));
    }

    Ok(DevServerCommandResponse {
        action: "exited".to_string(),
        url: context.url,
        scope: context.scope,
        repo_root: context.repo_root,
        repo_hash: context.repo_hash,
        leases,
        exit_code: status.code(),
        summary_mode: args.summary,
    })
}

async fn run_dev_server_list(
    args: &DevServerListArgs,
    base_url: &str,
) -> anyhow::Result<DevServerCommandResponse> {
    let context = build_dev_server_context(&args.output, base_url, "127.0.0.1", 0)?;
    let client = MemdClient::new(&context.base_url)?;
    let leases = client
        .dev_server_leases(&memd_schema::DevServerLeasesRequest {
            session: None,
            project: context.project.clone(),
            namespace: context.namespace.clone(),
            workspace: context.workspace.clone(),
            repo_hash: Some(context.repo_hash.clone()),
            active_only: Some(true),
            limit: Some(128),
        })
        .await?
        .leases;
    Ok(DevServerCommandResponse {
        action: "list".to_string(),
        url: String::new(),
        scope: String::new(),
        repo_root: context.repo_root,
        repo_hash: context.repo_hash,
        leases,
        exit_code: None,
        summary_mode: args.summary,
    })
}

async fn run_dev_server_release(
    args: &DevServerReleaseArgs,
    base_url: &str,
) -> anyhow::Result<DevServerCommandResponse> {
    let context = build_dev_server_context(&args.output, base_url, &args.host, args.port)?;
    let client = MemdClient::new(&context.base_url)?;
    let leases = client
        .release_dev_server_lease(&context.release_request())
        .await?
        .leases;
    Ok(DevServerCommandResponse {
        action: "release".to_string(),
        url: context.url,
        scope: context.scope,
        repo_root: context.repo_root,
        repo_hash: context.repo_hash,
        leases,
        exit_code: None,
        summary_mode: args.summary,
    })
}

pub(crate) fn render_dev_server_summary(response: &DevServerCommandResponse) -> String {
    let mut lines = vec![format!(
        "dev-server action={} url={} scope={} repo_hash={} leases={}",
        response.action,
        if response.url.is_empty() {
            "none"
        } else {
            response.url.as_str()
        },
        if response.scope.is_empty() {
            "none"
        } else {
            response.scope.as_str()
        },
        response.repo_hash,
        response.leases.len()
    )];
    for lease in &response.leases {
        lines.push(format!(
            "- {} holder={} url={} pid={} expires_at={}",
            lease.scope,
            lease
                .effective_agent
                .as_deref()
                .unwrap_or(lease.session.as_str()),
            lease.url,
            lease
                .pid
                .map(|pid| pid.to_string())
                .unwrap_or_else(|| "none".to_string()),
            lease.expires_at
        ));
    }
    lines.join("\n")
}

struct DevServerContext {
    base_url: String,
    scope: String,
    url: String,
    repo_root: String,
    repo_hash: String,
    session: String,
    tab_id: Option<String>,
    agent: Option<String>,
    effective_agent: Option<String>,
    project: Option<String>,
    namespace: Option<String>,
    workspace: Option<String>,
    host_name: Option<String>,
    host: String,
    port: u16,
}

impl DevServerContext {
    fn acquire_request(
        &self,
        command: &[String],
        pid: Option<u32>,
        ttl_seconds: u64,
        stale_after_seconds: u64,
    ) -> memd_schema::DevServerLeaseAcquireRequest {
        memd_schema::DevServerLeaseAcquireRequest {
            scope: self.scope.clone(),
            host: self.host.clone(),
            port: self.port,
            url: self.url.clone(),
            repo_root: self.repo_root.clone(),
            repo_hash: self.repo_hash.clone(),
            command: command.to_vec(),
            session: self.session.clone(),
            tab_id: self.tab_id.clone(),
            agent: self.agent.clone(),
            effective_agent: self.effective_agent.clone(),
            project: self.project.clone(),
            namespace: self.namespace.clone(),
            workspace: self.workspace.clone(),
            host_name: self.host_name.clone(),
            pid,
            ttl_seconds,
            recover_stale: true,
            stale_after_seconds,
        }
    }

    fn release_request(&self) -> memd_schema::DevServerLeaseReleaseRequest {
        memd_schema::DevServerLeaseReleaseRequest {
            scope: self.scope.clone(),
            session: self.session.clone(),
        }
    }
}

fn build_dev_server_context(
    output: &Path,
    base_url: &str,
    host: &str,
    port: u16,
) -> anyhow::Result<DevServerContext> {
    let runtime = read_bundle_runtime_config(output)?;
    let heartbeat = read_bundle_heartbeat(output)?;
    let session = runtime
        .as_ref()
        .and_then(|config| config.session.clone())
        .filter(|value| !value.trim().is_empty())
        .context("dev-server requires a configured bundle session")?;
    let agent = runtime.as_ref().and_then(|config| config.agent.clone());
    let effective_agent = runtime.as_ref().and_then(|config| {
        config
            .agent
            .as_deref()
            .map(|agent| compose_agent_identity(agent, config.session.as_deref()))
    });
    let repo_root = resolve_repo_root()?;
    let repo_hash = repo_hash(&repo_root);
    let host = host.trim().to_string();
    let scope = format!("resource:dev-server:{repo_hash}:{host}:{port}");
    let url = format!("http://{host}:{port}");
    Ok(DevServerContext {
        base_url: resolve_bundle_command_base_url(
            base_url,
            runtime
                .as_ref()
                .and_then(|config| config.base_url.as_deref()),
        ),
        scope,
        url,
        repo_hash,
        repo_root: repo_root.display().to_string(),
        session,
        tab_id: runtime.as_ref().and_then(|config| config.tab_id.clone()),
        agent,
        effective_agent,
        project: runtime.as_ref().and_then(|config| config.project.clone()),
        namespace: runtime.as_ref().and_then(|config| config.namespace.clone()),
        workspace: runtime.as_ref().and_then(|config| config.workspace.clone()),
        host_name: heartbeat.as_ref().and_then(|value| value.host.clone()),
        host,
        port,
    })
}

fn resolve_repo_root() -> anyhow::Result<PathBuf> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output();
    if let Ok(output) = output
        && output.status.success()
    {
        let root = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !root.is_empty() {
            return Ok(PathBuf::from(root));
        }
    }
    std::env::current_dir().context("resolve current dir as repo root")
}

fn repo_hash(root: &Path) -> String {
    let canonical = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let mut hasher = Sha256::new();
    hasher.update(canonical.display().to_string().as_bytes());
    let digest = hasher.finalize();
    digest[..8]
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>()
}

fn port_open(host: &str, port: u16) -> bool {
    (host, port)
        .to_socket_addrs()
        .ok()
        .and_then(|mut addrs| addrs.next())
        .is_some_and(|addr| TcpStream::connect_timeout(&addr, Duration::from_millis(250)).is_ok())
}

fn exit_status_error(status: ExitStatus, command: &[String]) -> anyhow::Error {
    anyhow::anyhow!(
        "dev server command exited with {}: {}",
        status,
        command.join(" ")
    )
}

fn refuse_cross_project_clawcontrol_dev_server(
    repo_root: &str,
    command: &[String],
) -> anyhow::Result<()> {
    if std::env::var("MEMD_ALLOW_CLAWCONTROL_DEV_SERVER").is_ok_and(|value| {
        let value = value.trim();
        value == "1" || value.eq_ignore_ascii_case("true")
    }) {
        return Ok(());
    }
    let repo_name = Path::new(repo_root)
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default();
    if repo_name == "clawcontrol" {
        return Ok(());
    }
    let command_text = command.join(" ").to_ascii_lowercase();
    if command_text.contains("clawcontrol") {
        anyhow::bail!(
            "refusing to launch ClawControl from {repo_root}; memd and ClawControl are separate. Run ClawControl from its own repo/session, or set MEMD_ALLOW_CLAWCONTROL_DEV_SERVER=1 for an intentional override."
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dev_server_guard_refuses_cross_project_clawcontrol_launch() {
        let err = refuse_cross_project_clawcontrol_dev_server(
            "/Volumes/T7/projects/memd",
            &[
                "bash".to_string(),
                "-lc".to_string(),
                "cd /Volumes/T7/projects/clawcontrol && cargo tauri dev".to_string(),
            ],
        )
        .expect_err("refuse cross-project launch");

        assert!(err.to_string().contains("refusing to launch ClawControl"));
    }

    #[test]
    fn dev_server_guard_allows_clawcontrol_repo_to_launch_itself() {
        refuse_cross_project_clawcontrol_dev_server(
            "/Volumes/T7/projects/clawcontrol",
            &["cargo".to_string(), "tauri".to_string(), "dev".to_string()],
        )
        .expect("same repo may launch itself");
    }
}
