use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::net::ToSocketAddrs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub server_url: String,
    #[serde(default)]
    pub token: String,
    #[serde(default = "default_interval")]
    pub interval_seconds: u32,
    #[serde(default = "default_heartbeat")]
    pub heartbeat_seconds: u32,
    #[serde(default = "default_idle")]
    pub idle_threshold_seconds: u32,
    #[serde(default)]
    pub enable_log: bool,
}

fn default_interval() -> u32 { 5 }
fn default_heartbeat() -> u32 { 60 }
fn default_idle() -> u32 { 300 }

impl Default for Config {
    fn default() -> Self {
        Self {
            server_url: String::new(),
            token: String::new(),
            interval_seconds: default_interval(),
            heartbeat_seconds: default_heartbeat(),
            idle_threshold_seconds: default_idle(),
            enable_log: false,
        }
    }
}

impl Config {
    fn normalize(&mut self) {
        self.server_url = self.server_url.trim().to_string();
        self.token = self.token.trim().to_string();
        self.interval_seconds = self.interval_seconds.clamp(1, 300);
        self.heartbeat_seconds = self.heartbeat_seconds.clamp(10, 600);
        self.idle_threshold_seconds = self.idle_threshold_seconds.clamp(30, 3600);
    }

    pub fn needs_setup(&self) -> bool {
        self.server_url.is_empty()
            || self.token.is_empty()
            || self.token == "YOUR_TOKEN_HERE"
    }
}

pub fn config_path() -> PathBuf {
    let exe = std::env::current_exe().unwrap_or_default();
    exe.parent().unwrap_or_else(|| std::path::Path::new(".")).join("config.json")
}

pub fn load() -> Config {
    let path = config_path();
    let text = match fs::read_to_string(&path) {
        Ok(t) => t,
        Err(_) => return Config::default(),
    };
    let mut cfg: Config = match serde_json::from_str(&text) {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("config.json 解析失败: {e}");
            return Config::default();
        }
    };
    cfg.normalize();
    cfg
}

pub fn save(cfg: &Config) -> Result<()> {
    let path = config_path();
    let dir = path.parent().context("config path has no parent")?;
    let data = serde_json::to_string_pretty(cfg).context("序列化失败")?;

    // Atomic write via temp file
    let tmp_path = dir.join(format!(".config_{}.tmp", std::process::id()));
    fs::write(&tmp_path, &data).context("写入临时文件失败")?;
    fs::rename(&tmp_path, &path).context("重命名配置文件失败")?;
    Ok(())
}

/// Return an error message string if invalid, None if valid.
pub fn validate(cfg: &Config) -> Option<String> {
    let url = cfg.server_url.trim();
    let token = cfg.token.trim();

    if url.is_empty() {
        return Some("服务器地址不能为空".into());
    }
    if token.is_empty() || token == "YOUR_TOKEN_HERE" {
        return Some("Token 不能为空".into());
    }

    let parsed = match url.parse::<url::Url>() {
        Ok(u) => u,
        Err(_) => return Some("服务器地址无效，请包含 http:// 或 https://".into()),
    };

    let scheme = parsed.scheme();
    if scheme != "http" && scheme != "https" {
        return Some("服务器地址必须使用 http:// 或 https://".into());
    }

    if let Some(host) = parsed.host_str() {
        if scheme == "http" {
            // Only allow private/internal addresses for plain HTTP
            let addr_str = format!("{}:80", host);
            if let Ok(mut addrs) = addr_str.to_socket_addrs() {
                if let Some(addr) = addrs.next() {
                    let ip = addr.ip();
                    if ip.is_global_unicast_or_public() {
                        return Some("HTTP 仅允许内网地址，公网请使用 HTTPS".into());
                    }
                }
            }
        }
    } else {
        return Some("服务器地址无效".into());
    }

    None
}

/// Extension trait for checking if an IP is globally routable.
trait IsPublic {
    fn is_global_unicast_or_public(&self) -> bool;
}

impl IsPublic for std::net::IpAddr {
    fn is_global_unicast_or_public(&self) -> bool {
        match self {
            std::net::IpAddr::V4(v4) => {
                !v4.is_private()
                    && !v4.is_loopback()
                    && !v4.is_link_local()
                    && !v4.is_broadcast()
                    && !v4.is_documentation()
                    && !v4.is_unspecified()
            }
            std::net::IpAddr::V6(v6) => {
                !v6.is_loopback()
                    && !v6.is_unspecified()
                    && (v6.segments()[0] & 0xfe00) != 0xfc00  // not ULA
                    && (v6.segments()[0] & 0xffc0) != 0xfe80  // not link-local
            }
        }
    }
}
