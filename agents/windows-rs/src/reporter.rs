use chrono::Utc;
use reqwest::Client;
use serde_json::{json, Value};
use std::time::{Duration, Instant};
use tracing::warn;

const MAX_BACKOFF_SECS: u64 = 60;
const PAUSE_AFTER_FAILURES: u32 = 5;
const PAUSE_DURATION_SECS: u64 = 300;

pub struct Reporter {
    endpoint: String,
    client: Client,
    consecutive_failures: u32,
    current_backoff_secs: u64,
    pause_until: Option<Instant>,
}

impl Reporter {
    pub fn new(server_url: &str, token: &str) -> Self {
        let endpoint = format!("{}/api/report", server_url.trim_end_matches('/'));
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .default_headers({
                let mut h = reqwest::header::HeaderMap::new();
                h.insert(
                    reqwest::header::AUTHORIZATION,
                    format!("Bearer {}", token).parse().unwrap(),
                );
                h.insert(
                    reqwest::header::CONTENT_TYPE,
                    "application/json".parse().unwrap(),
                );
                h
            })
            .build()
            .expect("Failed to build HTTP client");

        Self {
            endpoint,
            client,
            consecutive_failures: 0,
            current_backoff_secs: 0,
            pause_until: None,
        }
    }

    pub fn pause_remaining(&self) -> Option<Duration> {
        self.pause_until.and_then(|t| {
            let now = Instant::now();
            if t > now { Some(t - now) } else { None }
        })
    }

    pub fn retry_delay(&self) -> Option<Duration> {
        self.pause_remaining()
            .or_else(|| {
                if self.current_backoff_secs > 0 {
                    Some(Duration::from_secs(self.current_backoff_secs))
                } else {
                    None
                }
            })
    }

    pub async fn send(
        &mut self,
        app_id: &str,
        window_title: &str,
        extra: Option<Value>,
    ) -> bool {
        if self.pause_remaining().is_some() {
            return false;
        }

        let timestamp = Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();
        let title_trimmed: String = window_title.chars().take(256).collect();

        let mut payload = json!({
            "app_id": app_id,
            "window_title": title_trimmed,
            "timestamp": timestamp,
        });
        if let Some(extra_val) = extra {
            payload["extra"] = extra_val;
        }

        let result = self
            .client
            .post(&self.endpoint)
            .json(&payload)
            .send()
            .await;

        match result {
            Ok(resp) => {
                let status = resp.status().as_u16();
                if matches!(status, 200 | 201 | 409) {
                    self.consecutive_failures = 0;
                    self.current_backoff_secs = 0;
                    self.pause_until = None;
                    return true;
                }
                let body = resp.text().await.unwrap_or_default();
                warn!("Server {}: {}", status, &body[..body.len().min(200)]);
            }
            Err(e) => {
                warn!("请求失败: {e}");
            }
        }

        self.consecutive_failures += 1;
        self.current_backoff_secs = if self.current_backoff_secs == 0 {
            5
        } else {
            (self.current_backoff_secs * 2).min(MAX_BACKOFF_SECS)
        };

        if self.consecutive_failures >= PAUSE_AFTER_FAILURES {
            warn!(
                "连续失败 {} 次，暂停 {}s",
                self.consecutive_failures, PAUSE_DURATION_SECS
            );
            self.pause_until = Some(Instant::now() + Duration::from_secs(PAUSE_DURATION_SECS));
            self.consecutive_failures = 0;
            self.current_backoff_secs = 0;
        }

        false
    }
}

pub fn format_report_target(app_id: &str, window_title: &str) -> String {
    let app = app_id.trim();
    let title = window_title.trim();
    if title.is_empty() || title == app {
        return app.to_string();
    }
    let short_title: String = title.chars().take(80).collect();
    format!("{} — {}", app, short_title)
}
