use std::time::Duration;

use anyhow::{Context, Result};
use reqwest::blocking::Client;

pub struct NtfyNotifier {
    client: Client,
    endpoint: String,
    token: Option<String>,
}

impl NtfyNotifier {
    pub fn new(server: &str, topic: &str, token: Option<String>) -> Self {
        let endpoint = format!("{}/{}", server.trim_end_matches('/'), topic);
        Self {
            client: Client::new(),
            endpoint,
            token,
        }
    }

    pub fn send(&self, title: &str, body: &str) -> Result<()> {
        let mut request = self
            .client
            .post(&self.endpoint)
            .timeout(Duration::from_secs(15))
            .header("Title", title)
            .header("Priority", "high")
            .header("Tags", "robot,bell")
            .body(body.to_owned());
        if let Some(token) = &self.token {
            request = request.bearer_auth(token);
        }
        request
            .send()
            .context("failed to send ntfy notification")?
            .error_for_status()
            .context("ntfy returned an error status")?;
        Ok(())
    }
}
