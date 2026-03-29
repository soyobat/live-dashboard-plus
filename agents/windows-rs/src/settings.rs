use crate::config::{self, Config};
use eframe::egui;

/// Show the settings dialog (blocking). Returns new Config if saved, None if cancelled.
pub fn show(current: &Config) -> Option<Config> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Live Dashboard - 设置")
            .with_inner_size([460.0, 320.0])
            .with_resizable(false),
        ..Default::default()
    };

    eframe::run_native(
        "Live Dashboard - 设置",
        options,
        Box::new(|cc| {
            setup_custom_fonts(&cc);
            Ok(Box::new(SettingsApp::new(current.clone())))
        }),
    )
    .ok();

    // The app's result is communicated via a static/thread_local.
    // Use thread_local to pass result out of eframe.
    SETTINGS_RESULT.with(|r| r.borrow_mut().take())
}

thread_local! {
    static SETTINGS_RESULT: std::cell::RefCell<Option<Config>> = std::cell::RefCell::new(None);
}

fn setup_custom_fonts(cc: &eframe::CreationContext<'_>) {
    // Load system fonts that support Chinese characters
    let mut fonts = egui::FontDefinitions::default();
    
    // Try to load Windows system fonts
    let font_dirs = [
        "C:\\Windows\\Fonts",
    ];
    
    // Common Chinese fonts on Windows
    let chinese_fonts = [
        "simhei.ttf",      // 黑体
        "simsun.ttc",      // 宋体
        "simkai.ttf",      // 楷体
        "simfang.ttf",     // 仿宋
        "msyh.ttf",        // 微软雅黑
        "msyhbd.ttf",      // 微软雅黑 Bold
    ];
    
    for dir in &font_dirs {
        for font_name in &chinese_fonts {
            let font_path = std::path::Path::new(dir).join(font_name);
            if font_path.exists() {
                if let Ok(font_bytes) = std::fs::read(&font_path) {
                    fonts.font_data.insert(
                        "Chinese".to_owned(),
                        egui::FontData::from_owned(font_bytes),
                    );
                    fonts
                        .families
                        .get_mut(&egui::FontFamily::Proportional)
                        .unwrap()
                        .insert(0, "Chinese".to_owned());
                    fonts
                        .families
                        .get_mut(&egui::FontFamily::Monospace)
                        .unwrap()
                        .insert(0, "Chinese".to_owned());
                    cc.egui_ctx.set_fonts(fonts);
                    return;
                }
            }
        }
    }
}

struct SettingsApp {
    server_url: String,
    token: String,
    interval: u32,
    heartbeat: u32,
    idle_threshold: u32,
    enable_log: bool,
    error_msg: Option<String>,
    saved: bool,
}

impl SettingsApp {
    fn new(cfg: Config) -> Self {
        Self {
            server_url: cfg.server_url,
            token: cfg.token,
            interval: cfg.interval_seconds,
            heartbeat: cfg.heartbeat_seconds,
            idle_threshold: cfg.idle_threshold_seconds,
            enable_log: cfg.enable_log,
            error_msg: None,
            saved: false,
        }
    }

    fn to_config(&self) -> Config {
        Config {
            server_url: self.server_url.trim().to_string(),
            token: self.token.trim().to_string(),
            interval_seconds: self.interval,
            heartbeat_seconds: self.heartbeat,
            idle_threshold_seconds: self.idle_threshold,
            enable_log: self.enable_log,
        }
    }
}

impl eframe::App for SettingsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(8.0);

            egui::Grid::new("settings_grid")
                .num_columns(2)
                .spacing([8.0, 8.0])
                .min_col_width(120.0)
                .show(ui, |ui| {
                    ui.label("服务器地址:");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.server_url)
                            .desired_width(280.0)
                            .hint_text("https://your-domain.com"),
                    );
                    ui.end_row();

                    ui.label("Token:");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.token)
                            .desired_width(280.0)
                            .password(true)
                            .hint_text("YOUR_TOKEN_HERE"),
                    );
                    ui.end_row();

                    ui.label("上报间隔 (秒):");
                    ui.add(egui::Slider::new(&mut self.interval, 1..=300));
                    ui.end_row();

                    ui.label("心跳间隔 (秒):");
                    ui.add(egui::Slider::new(&mut self.heartbeat, 10..=600));
                    ui.end_row();

                    ui.label("AFK 判定 (秒):");
                    ui.add(egui::Slider::new(&mut self.idle_threshold, 30..=3600));
                    ui.end_row();

                    ui.label("开启日志文件:");
                    ui.checkbox(&mut self.enable_log, "保留 2 天");
                    ui.end_row();
                });

            ui.add_space(8.0);

            if let Some(err) = &self.error_msg {
                ui.colored_label(egui::Color32::RED, err);
                ui.add_space(4.0);
            }

            ui.horizontal(|ui| {
                if ui.button("  保存  ").clicked() {
                    let new_cfg = self.to_config();
                    if let Some(err) = config::validate(&new_cfg) {
                        self.error_msg = Some(err);
                    } else {
                        match config::save(&new_cfg) {
                            Ok(()) => {
                                self.saved = true;
                                SETTINGS_RESULT.with(|r| {
                                    *r.borrow_mut() = Some(new_cfg);
                                });
                                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                            }
                            Err(e) => {
                                self.error_msg = Some(format!("保存失败: {e}"));
                            }
                        }
                    }
                }
                if ui.button("  取消  ").clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });
        });
    }
}
