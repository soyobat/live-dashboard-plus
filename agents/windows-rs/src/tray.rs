use image::{Rgba, RgbaImage};
use parking_lot::Mutex;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tracing::{info, warn};
use tray_icon::{
    menu::{CheckMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem},
    Icon, TrayIconBuilder, TrayIconEvent,
};
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, PeekMessageW, TranslateMessage, MSG, PM_REMOVE,
};

/// Agent status — drives the tray icon color.
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum TrayStatus {
    Initializing,
    Online,
    Afk,
    Offline,
}

impl TrayStatus {
    fn color(&self) -> [u8; 3] {
        match self {
            TrayStatus::Online => [76, 175, 80],
            TrayStatus::Afk => [255, 152, 0],
            TrayStatus::Initializing | TrayStatus::Offline => [158, 158, 158],
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            TrayStatus::Initializing => "初始化中",
            TrayStatus::Online => "在线",
            TrayStatus::Afk => "AFK",
            TrayStatus::Offline => "离线",
        }
    }
}

/// State shared between the monitor task and the tray message pump.
pub struct TrayShared {
    pub status: TrayStatus,
    pub current_target: String,
    pub log_enabled: bool,
    pub quit_requested: bool,
    pub settings_requested: bool,
    /// Set by monitor task to trigger icon/tooltip refresh on main thread.
    pub needs_icon_update: bool,
}

impl TrayShared {
    pub fn new(log_enabled: bool) -> Self {
        Self {
            status: TrayStatus::Initializing,
            current_target: String::new(),
            log_enabled,
            quit_requested: false,
            settings_requested: false,
            needs_icon_update: false,
        }
    }
}

pub type SharedState = Arc<Mutex<TrayShared>>;

// ---------------------------------------------------------------------------
// Icon generation
// ---------------------------------------------------------------------------

fn make_icon(color: [u8; 3]) -> Icon {
    let size = 64u32;
    let mut img = RgbaImage::new(size, size);
    let cx = size as f32 / 2.0;
    let cy = cx;
    let radius = cx - 8.0;

    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let dx = x as f32 - cx;
        let dy = y as f32 - cy;
        *pixel = if dx * dx + dy * dy <= radius * radius {
            Rgba([color[0], color[1], color[2], 255])
        } else {
            Rgba([0, 0, 0, 0])
        };
    }
    Icon::from_rgba(img.into_raw(), size, size).expect("Failed to create tray icon")
}

// ---------------------------------------------------------------------------
// TrayAgentV2
// ---------------------------------------------------------------------------

pub struct TrayAgentV2 {
    shared: SharedState,
}

impl TrayAgentV2 {
    pub fn new(log_enabled: bool) -> Self {
        Self {
            shared: Arc::new(Mutex::new(TrayShared::new(log_enabled))),
        }
    }

    pub fn shared(&self) -> SharedState {
        Arc::clone(&self.shared)
    }

    /// Run the tray message pump. Blocks until quit or settings is requested.
    pub fn run(&self) {
        let log_enabled = self.shared.lock().log_enabled;

        // Build menu and capture all item IDs before tray creation
        let menu = Menu::new();
        let status_item = MenuItem::new("状态: 初始化中", false, None);
        let current_item = MenuItem::new("当前: 无", false, None);
        let sep1 = PredefinedMenuItem::separator();
        let log_item = CheckMenuItem::new("日志文件", true, log_enabled, None);
        let autostart_item =
            CheckMenuItem::new("开机自启", true, crate::autostart::is_enabled(), None);
        let settings_item = MenuItem::new("设置", true, None);
        let sep2 = PredefinedMenuItem::separator();
        let quit_item = MenuItem::new("退出", true, None);

        let _ = menu.append(&status_item);
        let _ = menu.append(&current_item);
        let _ = menu.append(&sep1);
        let _ = menu.append(&log_item);
        let _ = menu.append(&autostart_item);
        let _ = menu.append(&settings_item);
        let _ = menu.append(&sep2);
        let _ = menu.append(&quit_item);

        let log_id = log_item.id().clone();
        let autostart_id = autostart_item.id().clone();
        let settings_id = settings_item.id().clone();
        let quit_id = quit_item.id().clone();

        let tray = match TrayIconBuilder::new()
            .with_icon(make_icon(TrayStatus::Initializing.color()))
            .with_tooltip("Live Dashboard")
            .with_menu(Box::new(menu))
            .build()
        {
            Ok(t) => t,
            Err(e) => {
                warn!("托盘初始化失败: {e}");
                return;
            }
        };

        info!("系统托盘已启动");

        loop {
            // Process menu events
            while let Ok(event) = MenuEvent::receiver().try_recv() {
                let id = &event.id;
                if id == &quit_id {
                    info!("用户请求退出");
                    self.shared.lock().quit_requested = true;
                } else if id == &settings_id {
                    info!("用户打开设置");
                    self.shared.lock().settings_requested = true;
                } else if id == &log_id {
                    let new_enabled = !log_item.is_checked();
                    log_item.set_checked(new_enabled);
                    self.shared.lock().log_enabled = new_enabled;
                    info!("日志文件: {}", if new_enabled { "开启" } else { "关闭" });
                    let mut cfg = crate::config::load();
                    cfg.enable_log = new_enabled;
                    if let Err(e) = crate::config::save(&cfg) {
                        warn!("保存配置失败: {e}");
                    }
                } else if id == &autostart_id {
                    let currently = crate::autostart::is_enabled();
                    let want = !currently;
                    if crate::autostart::set_enabled(want) {
                        autostart_item.set_checked(want);
                    } else {
                        let msg = if want {
                            "无法开启开机自启，请检查当前账户权限。"
                        } else {
                            "关闭开机自启时未能清理全部启动项。\n请检查任务计划程序中的 LiveDashboardAgent。"
                        };
                        crate::win32::message_box("Live Dashboard", msg, true);
                        autostart_item.set_checked(currently);
                    }
                }
            }

            // Process tray icon events
            while let Ok(_event) = TrayIconEvent::receiver().try_recv() {}

            // Check stop condition
            {
                let state = self.shared.lock();
                if state.quit_requested || state.settings_requested {
                    break;
                }
            }

            // Update display if monitor task requested it
            {
                let needs = self.shared.lock().needs_icon_update;
                if needs {
                    let (color, label, current) = {
                        let s = self.shared.lock();
                        (s.status.color(), s.status.label(), s.current_target.clone())
                    };

                    let _ = tray.set_icon(Some(make_icon(color)));

                    let tooltip = {
                        let mut t = "Live Dashboard".to_string();
                        if !current.is_empty() {
                            let short: String = current.chars().take(50).collect();
                            t.push_str(&format!("\n当前: {}", short));
                        }
                        t.push_str(&format!("\n{}", label));
                        t.chars().take(127).collect::<String>()
                    };
                    let _ = tray.set_tooltip(Some(&tooltip));

                    status_item.set_text(&format!("状态: {}", label));
                    let current_display = if current.is_empty() {
                        "当前: 无".to_string()
                    } else {
                        format!("当前: {}", current.chars().take(60).collect::<String>())
                    };
                    current_item.set_text(&current_display);

                    self.shared.lock().needs_icon_update = false;
                }
            }

            // Pump Win32 messages (non-blocking)
            unsafe {
                let mut msg = MSG::default();
                while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
                    let _ = TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
            }

            thread::sleep(Duration::from_millis(50));
        }
    }
}
