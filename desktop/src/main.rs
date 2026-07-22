mod config;
mod provider;
mod chat;
mod theme;

use eframe::egui;
use std::sync::mpsc;
use std::thread;

/// Chat message in the UI
#[derive(Clone)]
struct Message {
    role: String,      // "user" | "assistant" | "progress" | "error"
    text: String,
}

pub struct App {
    messages: Vec<Message>,
    input: String,
    scroll_to_bottom: bool,
    config: config::Config,
    config_path: std::path::PathBuf,
    sending: bool,
    rx: Option<mpsc::Receiver<ChatEvent>>,
    sidebar_tab: SidebarTab,
    provider_list: Vec<provider::Provider>,
    // settings form
    form_provider: String,
    form_model: String,
    form_apikey: String,
    form_persona: String,
}

#[derive(Clone, PartialEq)]
enum SidebarTab {
    Chat,
    Settings,
    About,
}

enum ChatEvent {
    Progress(String),
    Reply(String),
    Error(String),
}

impl App {
    fn new() -> Self {
        let config_path = dirs::home_dir()
            .unwrap_or_default()
            .join(".angles")
            .join("config.json");
        let config = config::load_or_default(&config_path);
        let providers = provider::all_providers();
        let form_provider = config.provider.clone();
        let form_model = config.model.clone();
        let form_persona = config.agent_persona.clone();

        Self {
            messages: vec![Message {
                role: "assistant".into(),
                text: "Angles Desktop v0.1.0 — 已就绪。输入消息开始对话。".into(),
            }],
            input: String::new(),
            scroll_to_bottom: true,
            config,
            config_path,
            sending: false,
            rx: None,
            sidebar_tab: SidebarTab::Chat,
            provider_list: providers,
            form_provider,
            form_model,
            form_apikey: String::new(),
            form_persona,
        }
    }

    fn send_message(&mut self) {
        let text = self.input.trim().to_string();
        if text.is_empty() || self.sending { return; }

        self.messages.push(Message { role: "user".into(), text: text.clone() });
        self.input.clear();
        self.sending = true;
        self.scroll_to_bottom = true;

        // Collect history
        let history: Vec<chat::HistoryMessage> = self.messages.iter()
            .filter(|m| m.role == "user" || m.role == "assistant")
            .take(20)
            .map(|m| chat::HistoryMessage { role: m.role.clone(), content: m.text.clone() })
            .collect();

        let cfg = self.config.clone();
        let (tx, rx) = mpsc::channel();
        self.rx = Some(rx);

        thread::spawn(move || {
            let rt = match tokio::runtime::Runtime::new() {
                Ok(rt) => rt,
                Err(e) => { let _ = tx.send(ChatEvent::Error(e.to_string())); return; }
            };
            match rt.block_on(chat::exec_with_tools(&cfg, &text, &history)) {
                Ok(result) => {
                    for p in &result.progress {
                        let _ = tx.send(ChatEvent::Progress(p.clone()));
                    }
                    let _ = tx.send(ChatEvent::Reply(result.content));
                }
                Err(e) => { let _ = tx.send(ChatEvent::Error(e.to_string())); }
            }
        });
    }

    fn poll_messages(&mut self) {
        if let Some(rx) = &self.rx {
            while let Ok(event) = rx.try_recv() {
                match event {
                    ChatEvent::Progress(msg) => {
                        self.messages.push(Message { role: "progress".into(), text: msg });
                        self.scroll_to_bottom = true;
                    }
                    ChatEvent::Reply(msg) => {
                        if !msg.is_empty() {
                            self.messages.push(Message { role: "assistant".into(), text: msg });
                        }
                        self.sending = false;
                        self.scroll_to_bottom = true;
                    }
                    ChatEvent::Error(msg) => {
                        self.messages.push(Message { role: "error".into(), text: msg });
                        self.sending = false;
                        self.scroll_to_bottom = true;
                    }
                }
            }
        }
    }

    fn save_settings(&mut self) {
        let mut cfg = self.config.clone();
        cfg.provider = self.form_provider.clone();
        if let Some(p) = provider::all_providers().into_iter().find(|p| p.id == self.form_provider) {
            cfg.base_url = p.base_url;
            cfg.wire_api = p.wire_api;
        }
        cfg.model = self.form_model.clone();
        if !self.form_apikey.is_empty() { cfg.api_key = self.form_apikey.clone(); }
        cfg.agent_persona = self.form_persona.clone();
        let _ = config::save(&cfg, &self.config_path);
        self.config = cfg;
        self.form_apikey.clear();
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_messages();

        // Background
        let bg = egui::Color32::from_rgb(0, 0, 0);
        let _ = bg;

        // Top bar
        egui::TopBottomPanel::top("topbar").show(ctx, |ui| {
            ui.set_height(48.0);
            ui.horizontal_centered(|ui| {
                ui.add_space(12.0);
                ui.label(egui::RichText::new("α").color(theme::RED).font(egui::FontId::proportional(28.0)).strong());
                ui.label(egui::RichText::new("Angles").color(theme::TEXT).font(egui::FontId::proportional(20.0)).strong());
                ui.label(egui::RichText::new("Desktop").color(theme::MUTED).font(egui::FontId::proportional(13.0)));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(12.0);
                    let status = if self.config.api_key.is_empty() && std::env::var("ANGLES_API_KEY").is_err() {
                        "未配置"
                    } else {
                        &self.config.provider
                    };
                    ui.label(egui::RichText::new(status).color(theme::MUTED).font(egui::FontId::monospace(11.0)));
                    let (dot_color, _) = if self.sending { (theme::ORANGE, "busy") }
                        else if self.config.api_key.is_empty() { (theme::RED, "nokey") }
                        else { (theme::GREEN, "ok") };
                    ui.painter().circle_filled(ui.min_rect().right_center() - egui::vec2(80.0, 0.0), 4.0, dot_color);
                });
            });
        });

        // Bottom input bar
        egui::TopBottomPanel::bottom("inputbar").show(ctx, |ui| {
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.add_space(8.0);
                let send_enabled = !self.sending && !self.input.trim().is_empty();
                let response = ui.add(
                    egui::TextEdit::singleline(&mut self.input)
                        .hint_text("输入消息，Enter 发送...")
                        .font(egui::FontId::monospace(14.0))
                        .desired_width(f32::MAX)
                );
                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) && send_enabled {
                    self.send_message();
                }
                let btn = ui.add_enabled(send_enabled,
                    egui::Button::new(
                        egui::RichText::new("发送").color(egui::Color32::BLACK)
                    ).fill(theme::RED)
                );
                if btn.clicked() { self.send_message(); }
            });
            ui.add_space(6.0);
        });

        // Left sidebar
        egui::SidePanel::left("sidebar").resizable(false).default_width(180.0).show(ctx, |ui| {
            ui.set_min_height(ui.available_height());
            ui.add_space(12.0);
            ui.vertical(|ui| {
                let tabs = [
                    ("对话", SidebarTab::Chat),
                    ("设置", SidebarTab::Settings),
                    ("关于", SidebarTab::About),
                ];
                for (label, tab) in tabs {
                    let selected = self.sidebar_tab == tab;
                    let text = egui::RichText::new(label)
                        .color(if selected { theme::RED } else { theme::MUTED })
                        .font(egui::FontId::proportional(14.0))
                        .strong();
                ui.add(egui::Button::new(
                    egui::RichText::new(label)
                        .color(if selected { theme::RED } else { theme::MUTED })
                        .font(egui::FontId::proportional(14.0))
                        .strong()
                ));
                    if btn.clicked() { self.sidebar_tab = tab; }
                }
                ui.add_space(20.0);

                ui.label(egui::RichText::new(format!("Provider: {}", self.config.provider)).color(theme::MUTED).font(egui::FontId::monospace(10.0)));
                ui.label(egui::RichText::new(format!("Model: {}", self.config.model)).color(theme::MUTED).font(egui::FontId::monospace(10.0)));
                ui.add_space(10.0);
                ui.label(egui::RichText::new("v0.1.0").color(theme::MUTED_2).font(egui::FontId::monospace(10.0)));
            });
        });

        // Main content
        egui::CentralPanel::default().show(ctx, |ui| match self.sidebar_tab {
            SidebarTab::Chat => self.render_chat(ui),
            SidebarTab::Settings => self.render_settings(ui),
            SidebarTab::About => self.render_about(ui),
        });

        if self.scroll_to_bottom {
            self.scroll_to_bottom = false;
        }
    }
}

impl App {
    fn render_chat(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
            ui.add_space(8.0);
            for msg in &self.messages {
                ui.horizontal_wrapped(|ui| {
                    ui.add_space(12.0);
                    match msg.role.as_str() {
                        "user" => {
                            ui.label(egui::RichText::new(">").color(theme::RED).font(egui::FontId::monospace(13.0)).strong());
                            ui.label(egui::RichText::new(&msg.text).color(theme::TEXT).font(egui::FontId::proportional(14.0)));
                        }
                        "assistant" => {
                            ui.label(egui::RichText::new("α").color(theme::RED).font(egui::FontId::monospace(13.0)).strong());
                            ui.label(egui::RichText::new(&msg.text).color(theme::TEXT).font(egui::FontId::proportional(14.0)));
                        }
                        "progress" => {
                            ui.label(egui::RichText::new(&msg.text).color(theme::MUTED).font(egui::FontId::monospace(11.0)));
                        }
                        "error" => {
                            ui.label(egui::RichText::new(&msg.text).color(theme::RED).font(egui::FontId::monospace(12.0)));
                        }
                        _ => {}
                    }
                });
                ui.add_space(4.0);
            }
            if self.sending {
                ui.horizontal(|ui| {
                    ui.add_space(12.0);
                    ui.spinner();
                    ui.label(egui::RichText::new("思考中...").color(theme::MUTED).font(egui::FontId::monospace(12.0)));
                });
            }
            ui.add_space(8.0);
        });
    }

    fn render_settings(&mut self, ui: &mut egui::Ui) {
        ui.add_space(16.0);
        ui.heading(egui::RichText::new("设置").color(theme::TEXT).font(egui::FontId::proportional(22.0)).strong());
        ui.add_space(16.0);

        egui::Grid::new("settings").spacing([10.0, 12.0]).show(ui, |ui| {
            ui.label(egui::RichText::new("Provider").color(theme::MUTED).font(egui::FontId::monospace(12.0)));
            let mut selected = self.form_provider.clone();
            egui::ComboBox::from_id_salt("provider_combo")
                .selected_text(&selected)
                .show_ui(ui, |ui| {
                    for p in &self.provider_list {
                        ui.selectable_value(&mut selected, p.id.clone(), &p.name);
                    }
                });
            if selected != self.form_provider {
                self.form_provider = selected.clone();
                if let Some(p) = self.provider_list.iter().find(|p| p.id == selected) {
                    self.form_model = p.default_model.clone();
                }
            }
            ui.end_row();

            ui.label(egui::RichText::new("Model").color(theme::MUTED).font(egui::FontId::monospace(12.0)));
            ui.text_edit_singleline(&mut self.form_model)
                .on_hover_text("模型 ID");
            ui.end_row();

            ui.label(egui::RichText::new("API Key").color(theme::MUTED).font(egui::FontId::monospace(12.0)));
            ui.add(
                egui::TextEdit::singleline(&mut self.form_apikey)
                    .password(true)
                    .hint_text("留空保留现有 key")
            );
            ui.end_row();

            ui.label(egui::RichText::new("人设").color(theme::MUTED).font(egui::FontId::monospace(12.0)));
            ui.add(
                egui::TextEdit::multiline(&mut self.form_persona)
                    .desired_width(300.0)
                    .desired_rows(2)
            );
            ui.end_row();
        });

        ui.add_space(20.0);
        if ui.add(egui::Button::new(egui::RichText::new("保存").color(egui::Color32::BLACK)).fill(theme::RED)).clicked() {
            self.save_settings();
            self.messages.push(Message {
                role: "progress".into(),
                text: "配置已保存".into(),
            });
        }

        ui.add_space(20.0);
        ui.label(egui::RichText::new(format!("配置文件: {}", self.config_path.display()))
            .color(theme::MUTED_2)
            .font(egui::FontId::monospace(10.0)));
    }

    fn render_about(&mut self, ui: &mut egui::Ui) {
        ui.add_space(40.0);
        ui.vertical_centered(|ui| {
            ui.label(egui::RichText::new("α").color(theme::RED).font(egui::FontId::proportional(60.0)).strong());
            ui.add_space(8.0);
            ui.label(egui::RichText::new("Angles Desktop").color(theme::TEXT).font(egui::FontId::proportional(24.0)).strong());
            ui.label(egui::RichText::new("v0.1.0").color(theme::MUTED).font(egui::FontId::monospace(13.0)));
            ui.add_space(20.0);
            ui.label(egui::RichText::new("开源 AI 编程智能体 — 桌面版").color(theme::MUTED));
            ui.label(egui::RichText::new("Rust + egui, MIT License").color(theme::MUTED_2).font(egui::FontId::monospace(11.0)));
            ui.add_space(20.0);
            ui.label(egui::RichText::new("github.com/ZSJ305/angles-cli").color(theme::RED).font(egui::FontId::monospace(12.0)));
        });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 700.0])
            .with_min_inner_size([600.0, 400.0])
            .with_title("Angles Desktop — AI Coding Agent"),
        ..Default::default()
    };
    eframe::run_native(
        "Angles Desktop",
        options,
        Box::new(|cc| {
            // Force dark theme
            let ctx = &cc.egui_ctx;
            let mut visuals = egui::Visuals::dark();
            visuals.panel_fill = egui::Color32::from_rgb(0, 0, 0);
            visuals.window_fill = egui::Color32::from_rgb(17, 17, 17);
            visuals.extreme_bg_color = egui::Color32::from_rgb(0, 0, 0);
            visuals.faint_bg_color = egui::Color32::from_rgb(10, 10, 10);
            ctx.set_visuals(visuals);
            Ok(Box::new(App::new()))
        }),
    )
}
