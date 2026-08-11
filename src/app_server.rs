use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use anyhow::{Context, Result, anyhow, bail};
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuotaWindow {
    pub used_percent: f64,
    pub window_duration_mins: u64,
    pub resets_at: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LimitSet {
    primary: Option<QuotaWindow>,
    secondary: Option<QuotaWindow>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RateLimitsResult {
    rate_limits: Option<LimitSet>,
    rate_limits_by_limit_id: Option<std::collections::HashMap<String, LimitSet>>,
}

pub struct AppServer {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    next_id: u64,
}

impl AppServer {
    pub fn start(command: &str) -> Result<Self> {
        let mut child = Command::new(command)
            .args(["app-server", "--stdio"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .with_context(|| format!("failed to start `{command} app-server --stdio`"))?;
        let stdin = child.stdin.take().context("missing app-server stdin")?;
        let stdout = child.stdout.take().context("missing app-server stdout")?;
        let mut this = Self {
            child,
            stdin,
            stdout: BufReader::new(stdout),
            next_id: 1,
        };
        this.initialize()?;
        Ok(this)
    }

    fn initialize(&mut self) -> Result<()> {
        let id = self.next_request_id();
        self.send(&json!({
            "method": "initialize",
            "id": id,
            "params": {"clientInfo": {
                "name": "codex_reset_notifier",
                "title": "Codex Reset Notifier",
                "version": env!("CARGO_PKG_VERSION")
            }}
        }))?;
        self.read_response(id)?;
        self.send(&json!({"method": "initialized"}))?;
        Ok(())
    }

    pub fn read_quota_windows(&mut self) -> Result<Vec<QuotaWindow>> {
        let id = self.next_request_id();
        self.send(&json!({"method": "account/rateLimits/read", "id": id}))?;
        let value = self.read_response(id)?;
        let result: RateLimitsResult = serde_json::from_value(
            value
                .get("result")
                .cloned()
                .context("missing rate-limit result")?,
        )?;

        let limits = result
            .rate_limits_by_limit_id
            .as_ref()
            .and_then(|m| m.get("codex"))
            .or(result.rate_limits.as_ref())
            .context("Codex rate limits are unavailable")?;

        let mut windows = Vec::new();
        if let Some(window) = limits.primary.clone() {
            windows.push(window);
        }
        if let Some(window) = limits.secondary.clone() {
            windows.push(window);
        }
        Ok(windows)
    }

    fn next_request_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    fn send(&mut self, value: &Value) -> Result<()> {
        serde_json::to_writer(&mut self.stdin, value)?;
        self.stdin.write_all(b"\n")?;
        self.stdin.flush()?;
        Ok(())
    }

    fn read_response(&mut self, expected_id: u64) -> Result<Value> {
        loop {
            let mut line = String::new();
            let bytes = self.stdout.read_line(&mut line)?;
            if bytes == 0 {
                bail!("codex app-server closed stdout");
            }
            let value: Value = serde_json::from_str(line.trim())
                .with_context(|| format!("invalid app-server JSON: {}", line.trim()))?;
            if value.get("id").and_then(Value::as_u64) != Some(expected_id) {
                continue;
            }
            if let Some(error) = value.get("error") {
                return Err(anyhow!("app-server error: {error}"));
            }
            return Ok(value);
        }
    }
}

impl Drop for AppServer {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
