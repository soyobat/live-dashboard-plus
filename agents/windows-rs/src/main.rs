#![windows_subsystem = "windows"]

mod audio;
mod autostart;
mod config;
mod monitor;
mod music;
mod reporter;
mod settings;
mod tray;
mod win32;

use std::sync::Arc;
use tokio_util::sync::CancellationToken;
use tracing::info;

fn main() {
    // Console logging always on (no file logging for simplicity; toggle saved to config)
    tracing_subscriber::fmt()
        .with_ansi(false)
        .with_target(false)
        .with_writer(std::io::stderr)
        .init();

    info!("Live Dashboard Windows Agent (Rust)");

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to build tokio runtime");

    loop {
        let mut cfg = config::load();

        // Show settings dialog if config is not ready
        if cfg.needs_setup() {
            match settings::show(&cfg) {
                Some(_) => {}
                None => {
                    info!("设置取消，退出");
                    return;
                }
            }
            cfg = config::load();
        }

        if let Some(err) = config::validate(&cfg) {
            tracing::warn!("配置无效: {err}");
            match settings::show(&cfg) {
                Some(_) => {
                    cfg = config::load();
                }
                None => return,
            }
            continue;
        }

        info!("连接服务器: {}", cfg.server_url);

        // Initialize tray agent
        let tray_agent = tray::TrayAgentV2::new(cfg.enable_log);
        let shared = tray_agent.shared();

        // Launch monitor task
        let cancel = CancellationToken::new();
        let cancel_clone = cancel.clone();
        let shared_clone = Arc::clone(&shared);
        let cfg_clone = cfg.clone();

        let monitor_handle = rt.spawn(async move {
            monitor::run(cfg_clone, shared_clone, cancel_clone).await;
        });

        // Run tray (blocks main thread via Win32 message pump)
        tray_agent.run();

        // Tray exited — stop monitor
        cancel.cancel();
        rt.block_on(async {
            let _ = tokio::time::timeout(
                std::time::Duration::from_secs(5),
                monitor_handle,
            )
            .await;
        });

        let (quit_req, settings_req) = {
            let s = shared.lock();
            (s.quit_requested, s.settings_requested)
        };

        if quit_req {
            info!("Agent 退出");
            break;
        }

        if settings_req {
            settings::show(&cfg);
            // Reload and restart loop
            continue;
        }
    }
}
