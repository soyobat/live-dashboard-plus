use crate::{
    audio, config::Config, music, reporter::{format_report_target, Reporter},
    tray::{SharedState, TrayStatus}, win32,
};
use serde_json::{json, Value};
use std::time::{Duration, Instant};
use tokio_util::sync::CancellationToken;
use tracing::info;

pub async fn run(cfg: Config, shared: SharedState, cancel: CancellationToken) {
    let interval = Duration::from_secs(cfg.interval_seconds as u64);
    let heartbeat_interval = Duration::from_secs(cfg.heartbeat_seconds as u64);
    let idle_threshold = cfg.idle_threshold_seconds as f64;

    let mut reporter = Reporter::new(&cfg.server_url, &cfg.token);

    let mut prev_app: Option<String> = None;
    let mut prev_title: Option<String> = None;
    let mut last_report_time = Instant::now() - heartbeat_interval; // report immediately on start
    let mut was_idle = false;

    info!(
        "监控启动 — 上报间隔={}s, 心跳={}s, AFK判定={}s",
        cfg.interval_seconds, cfg.heartbeat_seconds, cfg.idle_threshold_seconds
    );

    loop {
        // Check for cancellation
        if cancel.is_cancelled() {
            break;
        }

        let now = Instant::now();

        // AFK detection
        let idle_secs = win32::get_idle_secs();
        let is_idle = idle_secs >= idle_threshold
            && !audio::is_audio_playing()
            && !win32::is_fullscreen();

        if is_idle && !was_idle {
            info!("用户离开 (空闲 {:.0}s)", idle_secs);
            was_idle = true;
            update_status(&shared, TrayStatus::Afk, None);
        } else if !is_idle && was_idle {
            info!("用户回归");
            was_idle = false;
        }

        if is_idle {
            let heartbeat_due = now.duration_since(last_report_time) >= heartbeat_interval;
            if heartbeat_due {
                let extra = battery_extra();
                let success = reporter.send("idle", "User is away", Some(extra)).await;
                if success {
                    prev_app = Some("idle".into());
                    prev_title = Some("User is away".into());
                    last_report_time = now;
                    let target = format_report_target("idle", "User is away");
                    update_status(&shared, TrayStatus::Afk, Some(target));
                } else if let Some(delay) = reporter.retry_delay() {
                    tokio::select! {
                        _ = tokio::time::sleep(delay) => {}
                        _ = cancel.cancelled() => break,
                    }
                    continue;
                }
            }
            tokio::select! {
                _ = tokio::time::sleep(interval) => {}
                _ = cancel.cancelled() => break,
            }
            continue;
        }

        // Active: get foreground window
        let info = win32::get_foreground_info();
        let Some((app_id, title)) = info else {
            tokio::select! {
                _ = tokio::time::sleep(interval) => {}
                _ = cancel.cancelled() => break,
            }
            continue;
        };

        update_status(&shared, TrayStatus::Online, None);

        let changed = prev_app.as_deref() != Some(app_id.as_str())
            || prev_title.as_deref() != Some(title.as_str());
        let heartbeat_due = now.duration_since(last_report_time) >= heartbeat_interval;

        if changed || heartbeat_due {
            let mut extra = battery_extra();
            if let Some(music) = music::get_music_info() {
                if let Ok(v) = serde_json::to_value(&music) {
                    extra["music"] = v;
                }
            }

            let target = format_report_target(&app_id, &title);
            let success = reporter.send(&app_id, &title, Some(extra)).await;
            if success {
                prev_app = Some(app_id);
                prev_title = Some(title);
                last_report_time = now;
                if changed {
                    info!("上报: {}", target);
                }
                update_status(&shared, TrayStatus::Online, Some(target));
            } else if let Some(delay) = reporter.retry_delay() {
                tokio::select! {
                    _ = tokio::time::sleep(delay) => {}
                    _ = cancel.cancelled() => break,
                }
                continue;
            }
        }

        tokio::select! {
            _ = tokio::time::sleep(interval) => {}
            _ = cancel.cancelled() => break,
        }
    }

    info!("监控停止");
}

fn battery_extra() -> Value {
    match win32::get_battery() {
        Some(b) => json!({
            "battery_percent": b.percent,
            "battery_charging": b.charging,
        }),
        None => json!({}),
    }
}

fn update_status(shared: &SharedState, status: TrayStatus, current_target: Option<String>) {
    let mut state = shared.lock();
    state.status = status;
    if let Some(t) = current_target {
        state.current_target = t;
    }
    state.needs_icon_update = true;
}
