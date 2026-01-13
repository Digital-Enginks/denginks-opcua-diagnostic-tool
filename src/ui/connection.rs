



use eframe::egui;
use std::sync::mpsc;
use tokio::runtime::Handle;

use crate::app::BackendMessage;
use crate::config::bookmarks::{AuthMethod, Bookmarks, MessageSecurityMode, SecurityPolicy, ServerBookmark};
use crate::network::diagnostics::{DiagnosticResult, DiagnosticStep, StepStatus};
use crate::network::discovery::EndpointInfo;
use crate::opcua::client::ClientConfig;
use crate::opcua::certificates::CertificateManager;
use crate::utils::i18n::{self, T, Language};


pub enum ConnectionAction {
    Connect(ClientConfig),
    Disconnect,
    StartDiagnostic(String),
    CancelDiagnostic,
}


pub struct ConnectionPanel {
    
    server_input: String,
    
    security_policy: SecurityPolicy,
    
    security_mode: MessageSecurityMode,
    
    use_auth: bool,
    
    username: String,
    
    password: String,
    
    bookmark_name: String,
    
    show_add_bookmark: bool,

    
    is_connecting: bool,
    
    
    
    is_diagnosing: bool,
    
    diagnostic_log: Vec<DiagnosticStep>,
    
    diagnostic_result: Option<DiagnosticResult>,
    
    discovered_endpoints: Vec<EndpointInfo>,
    
    selected_endpoint: Option<usize>,
    
    diagnostic_start: Option<std::time::Instant>,
}

impl Default for ConnectionPanel {
    fn default() -> Self {
        Self {
            server_input: String::new(),
            security_policy: SecurityPolicy::None,
            security_mode: MessageSecurityMode::None,
            use_auth: false,
            username: String::new(),
            password: String::new(),
            bookmark_name: String::new(),
            show_add_bookmark: false,

            is_connecting: false,
            is_diagnosing: false,
            diagnostic_log: Vec::new(),
            diagnostic_result: None,
            discovered_endpoints: Vec::new(),
            selected_endpoint: None,
            diagnostic_start: None,
        }
    }
}

impl ConnectionPanel {
    
    pub fn add_diagnostic_step(&mut self, step: DiagnosticStep) {
        
        if let Some(existing) = self.diagnostic_log.iter_mut().find(|s| s.id == step.id) {
            *existing = step;
        } else {
            self.diagnostic_log.push(step);
        }
    }

    
    pub fn set_diagnostic_result(&mut self, result: DiagnosticResult) {
        self.is_diagnosing = false;
        self.discovered_endpoints = result.endpoints.clone();
        self.diagnostic_result = Some(result);
        self.diagnostic_start = None;
    }

    
    pub fn reset_diagnostic(&mut self) {
        self.is_diagnosing = false;
        self.diagnostic_log.clear();
        self.diagnostic_result = None;
        self.discovered_endpoints.clear();
        self.selected_endpoint = None;
        self.diagnostic_start = None;
    }

    
    pub fn start_diagnostic(&mut self) {
        self.is_diagnosing = true;
        self.diagnostic_log.clear();
        self.diagnostic_result = None;
        self.diagnostic_start = Some(std::time::Instant::now());
    }

    
    pub fn set_connecting(&mut self, connecting: bool) {
        self.is_connecting = connecting;
    }

    
    fn is_interactive(&self, is_connected: bool, app_busy: bool) -> bool {
        !is_connected && !app_busy && !self.is_connecting && !self.is_diagnosing
    }

    
    fn get_elapsed_str(&self) -> Option<String> {
        self.diagnostic_start.map(|start| {
            let elapsed = start.elapsed().as_secs();
            format!("{}s", elapsed)
        })
    }

    
    #[allow(clippy::too_many_arguments)]
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        bookmarks: &mut Bookmarks,
        display_elapsed: Option<String>,
        can_cancel: bool,
        _runtime: &Handle,
        _backend_tx: mpsc::Sender<BackendMessage>,
        is_connected: bool,
        app_busy: bool,
        lang: Language,
    ) -> (Option<ConnectionAction>, bool) {
        let mut action: Option<ConnectionAction> = None;
        let mut should_disconnect = false;

        ui.heading(format!("🔌 {}", i18n::t(T::Connection, lang)));
        ui.separator();

        
        if is_connected {
            ui.add_space(5.0);
            if ui.button(format!("🔌 {}", i18n::t(T::Disconnect, lang)))
                .on_hover_text("Terminates the current OPC UA session")
                .clicked() 
            {
                should_disconnect = true;
            }
            ui.add_space(10.0);
            ui.separator();
        }

        
        egui::CollapsingHeader::new(format!("📚 {}", i18n::t(T::SavedServers, lang)))
            .default_open(!is_connected)
            .show(ui, |ui| {
                self.show_bookmarks(ui, bookmarks, lang);
            });

        ui.add_space(10.0);

        
        egui::CollapsingHeader::new(format!("➕ {}", i18n::t(T::NewConnection, lang)))
            .default_open(!is_connected)
            .show(ui, |ui| {
                action = self.show_new_connection(ui, bookmarks, display_elapsed, can_cancel, is_connected, app_busy, lang);
            });

        if should_disconnect {
            (Some(ConnectionAction::Disconnect), false)
        } else {
            (action, false)
        }
    }

    fn show_bookmarks(&mut self, ui: &mut egui::Ui, bookmarks: &mut Bookmarks, lang: Language) {
        if bookmarks.is_empty() {
            ui.label(i18n::t(T::NoSavedServers, lang));
        } else {
            let mut to_remove: Option<usize> = None;
            let mut to_load: Option<usize> = None;

            for (i, bookmark) in bookmarks.servers.iter().enumerate() {
                ui.horizontal(|ui| {
                    if ui.button("📂").on_hover_text(i18n::t(T::LoadBookmark, lang)).clicked() {
                        to_load = Some(i);
                    }
                    if ui.button("🗑").on_hover_text(i18n::t(T::DeleteBookmark, lang)).clicked() {
                        to_remove = Some(i);
                    }
                    ui.label(&bookmark.name);
                });
                ui.label(format!("  {}", bookmark.endpoint_url));
                ui.add_space(4.0);
            }

            
            if let Some(idx) = to_remove {
                bookmarks.remove(idx);
                let _ = bookmarks.save();
            }

            
            if let Some(idx) = to_load {
                if let Some(bookmark) = bookmarks.servers.get(idx) {
                    self.server_input = bookmark.endpoint_url.clone();
                    self.security_policy = bookmark.security_policy.clone();
                    self.security_mode = bookmark.security_mode.clone();
                    match &bookmark.auth_method {
                        AuthMethod::Anonymous => {
                            self.use_auth = false;
                            self.username.clear();
                            self.password.clear();
                        }
                        AuthMethod::UserPassword { username, password } => {
                            self.use_auth = true;
                            self.username = username.clone();
                            self.password = password.clone();
                        }
                    }
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn show_new_connection(
        &mut self,
        ui: &mut egui::Ui,
        bookmarks: &mut Bookmarks,
        _display_elapsed: Option<String>,
        can_cancel: bool,
        is_connected: bool,
        app_busy: bool,
        lang: Language,
    ) -> Option<ConnectionAction> {
        let mut action: Option<ConnectionAction> = None;
        let interactive = self.is_interactive(is_connected, app_busy);

        
        ui.horizontal(|ui| {
            ui.label(i18n::t(T::ServerInput, lang));
        });
        
        let text_response = ui.add_enabled(
            interactive,
            egui::TextEdit::singleline(&mut self.server_input)
                .hint_text("192.168.1.100 or opc.tcp://server:4840")
                .desired_width(ui.available_width() - 10.0)
        );

        
        if text_response.lost_focus()
            && ui.input(|i| i.key_pressed(egui::Key::Enter))
            && !self.server_input.is_empty()
            && interactive
        {
            self.start_diagnostic();
            action = Some(ConnectionAction::StartDiagnostic(self.server_input.clone()));
        }

        ui.add_space(5.0);

        
        ui.horizontal(|ui| {
            if self.is_diagnosing {
                ui.spinner();
                let elapsed = self.get_elapsed_str().unwrap_or_default();
                ui.label(format!("{} ({})", i18n::t(T::Diagnose, lang), elapsed));
                
                
                if ui.button(format!("⏹ {}", i18n::t(T::Stop, lang)))
                    .on_hover_text(i18n::t(T::CancelTask, lang))
                    .clicked() 
                {
                    self.reset_diagnostic();
                    action = Some(ConnectionAction::CancelDiagnostic);
                }
            } else {
                let diagnose_enabled = !self.server_input.is_empty() && interactive;
                if ui.add_enabled(diagnose_enabled, egui::Button::new(format!("🔍 {}", i18n::t(T::Diagnose, lang))))
                    .on_hover_text("Validates input, resolves DNS, scans ports, and discovers endpoints")
                    .clicked() 
                {
                    self.start_diagnostic();
                    action = Some(ConnectionAction::StartDiagnostic(self.server_input.clone()));
                }
            }
        });

        
        if self.is_diagnosing || !self.diagnostic_log.is_empty() {
            ui.add_space(5.0);
            ui.label(egui::RichText::new(i18n::t(T::DiagnosticLog, lang)).strong());
            
            egui::Frame::dark_canvas(ui.style())
                .inner_margin(egui::Margin::same(8))
                .show(ui, |ui| {
                    
                    if self.diagnostic_log.is_empty() && self.is_diagnosing {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label(egui::RichText::new("Initializing diagnostic...").color(egui::Color32::from_rgb(100, 200, 255)));
                        });
                    }
                    
                    egui::ScrollArea::vertical()
                        .max_height(120.0)
                        .show(ui, |ui| {
                            for step in &self.diagnostic_log {
                                ui.horizontal(|ui| {
                                    let color = match step.status {
                                        StepStatus::Success => egui::Color32::from_rgb(100, 255, 100),
                                        StepStatus::Warning => egui::Color32::from_rgb(255, 200, 100),
                                        StepStatus::Failed => egui::Color32::from_rgb(255, 100, 100),
                                        StepStatus::Running => egui::Color32::from_rgb(100, 200, 255),
                                        StepStatus::Pending => egui::Color32::GRAY,
                                    };
                                    
                                    ui.label(egui::RichText::new(step.status.icon()).color(color));
                                    ui.label(&step.name);
                                    
                                    if step.duration_ms > 0 {
                                        ui.label(egui::RichText::new(format!("({}ms)", step.duration_ms)).weak());
                                    } else if step.status == StepStatus::Running {
                                        ui.spinner();
                                    }
                                });
                                
                                if !step.details.is_empty() {
                                    ui.indent("detail", |ui| {
                                        ui.label(egui::RichText::new(&step.details).small().weak());
                                    });
                                }
                            }
                        });
                });
        }

        
        if !self.discovered_endpoints.is_empty() {
            ui.add_space(5.0);
            ui.label(egui::RichText::new(
                i18n::t(T::FoundEndpoints, lang).replace("{}", &self.discovered_endpoints.len().to_string())
            ).strong());
            
            egui::ScrollArea::vertical().max_height(120.0).show(ui, |ui| {
                for (i, ep) in self.discovered_endpoints.iter().enumerate() {
                    let selected = self.selected_endpoint == Some(i);
                    if ui.add_enabled(interactive, egui::Button::new(ep.display_name(lang)).selected(selected)).clicked() {
                        self.selected_endpoint = Some(i);
                        
                        
                        self.security_policy = match ep.security_policy_name.as_str() {
                            "None" => SecurityPolicy::None,
                            "Basic128Rsa15" => SecurityPolicy::Basic128Rsa15,
                            "Basic256" => SecurityPolicy::Basic256,
                            "Basic256Sha256" => SecurityPolicy::Basic256Sha256,
                            "Aes128Sha256RsaOaep" | "Aes128-Sha256-RsaOaep" => SecurityPolicy::Aes128Sha256RsaOaep,
                            "Aes256Sha256RsaPss" | "Aes256-Sha256-RsaPss" => SecurityPolicy::Aes256Sha256RsaPss,
                            _ => SecurityPolicy::None,
                        };
                        
                        self.security_mode = match ep.security_mode.as_str() {
                            "None" => MessageSecurityMode::None,
                            "Sign" => MessageSecurityMode::Sign,
                            _ => MessageSecurityMode::SignAndEncrypt,
                        };
                        
                        
                        self.use_auth = !ep.allows_anonymous();
                    }
                }
            });
        }
        
        
        if let Some(result) = &self.diagnostic_result {
            ui.add_space(5.0);
            if result.overall_success {
                ui.colored_label(
                    egui::Color32::from_rgb(100, 255, 100),
                    format!("✅ {} ({}ms)", i18n::t(T::DiagnosticComplete, lang), result.total_duration_ms)
                );
                if let Some(url) = &result.recommended_url {
                    
                    if self.server_input != *url && !url.is_empty() {
                        ui.horizontal(|ui| {
                            ui.label("→");
                            if ui.link(url).clicked() {
                                self.server_input = url.clone();
                            }
                        });
                    }
                }
            } else {
                ui.colored_label(
                    egui::Color32::from_rgb(255, 100, 100),
                    format!("❌ {}", i18n::t(T::DiagnosticFailed, lang))
                );
            }
        }

        ui.add_space(5.0);
        ui.separator();

        
        let security_locked = self.selected_endpoint.is_some();
        
        ui.horizontal(|ui| {
            ui.label(i18n::t(T::SecurityPolicy, lang));
            if security_locked {
                ui.label(egui::RichText::new(self.security_policy.display_name(lang)).strong());
                ui.label(egui::RichText::new("🔒").small());
            } else {
                egui::ComboBox::from_id_salt("security_policy")
                    .selected_text(self.security_policy.display_name(lang))
                    .show_ui(ui, |ui| {
                        if is_connected {
                            ui.disable();
                        }
                        for policy in SecurityPolicy::all() {
                            ui.selectable_value(
                                &mut self.security_policy,
                                policy.clone(),
                                policy.display_name(lang),
                            );
                        }
                    });
            }
        });

        ui.horizontal(|ui| {
            ui.label(i18n::t(T::SecurityMode, lang));
            if security_locked {
                ui.label(egui::RichText::new(self.security_mode.display_name(lang)).strong());
                ui.label(egui::RichText::new("🔒").small());
            } else {
                egui::ComboBox::from_id_salt("security_mode")
                    .selected_text(self.security_mode.display_name(lang))
                    .show_ui(ui, |ui| {
                        if is_connected {
                            ui.disable();
                        }
                        for mode in MessageSecurityMode::all() {
                            ui.selectable_value(
                                &mut self.security_mode,
                                mode.clone(),
                                mode.display_name(lang),
                            );
                        }
                    });
            }
        });

        ui.add_space(5.0);

        
        ui.add_enabled(interactive && !security_locked, egui::Checkbox::new(&mut self.use_auth, i18n::t(T::UseAuth, lang)));
        if self.use_auth {
            ui.horizontal(|ui| {
                ui.label(i18n::t(T::Username, lang));
                ui.add_enabled(interactive, egui::TextEdit::singleline(&mut self.username));
            });
            ui.horizontal(|ui| {
                ui.label(i18n::t(T::Password, lang));
                ui.add_enabled(interactive, egui::TextEdit::singleline(&mut self.password).password(true));
            });
        }

        ui.add_space(10.0);
        ui.separator();

        
        ui.horizontal(|ui| {
            let connect_enabled = !self.server_input.is_empty() && interactive;
            
            if self.is_connecting {
                ui.spinner();
                ui.label(i18n::t(T::Connecting, lang));
                
                if can_cancel && ui.button(format!("⏹ {}", i18n::t(T::Stop, lang)))
                        .on_hover_text(i18n::t(T::CancelTask, lang))
                        .clicked() 
                 {
                        self.is_connecting = false;
                        
                 }

            } else if ui.add_enabled(connect_enabled, egui::Button::new(format!("🔗 {}", i18n::t(T::Connect, lang))))
                .on_hover_text("Establishes a secure OPC UA session")
                .clicked() 
            {
                let _pki_dir = CertificateManager::new()
                    .map(|m| m.pki_directory().to_path_buf())
                    .unwrap_or_else(|_| std::path::PathBuf::from("./pki"));

                let auth_method = if self.use_auth {
                    AuthMethod::UserPassword {
                        username: self.username.clone(),
                        password: self.password.clone(),
                    }
                } else {
                    AuthMethod::Anonymous
                };

                
                let endpoint_url = self.diagnostic_result
                    .as_ref()
                    .and_then(|r| r.recommended_url.clone())
                    .unwrap_or_else(|| {
                        
                        if self.server_input.starts_with("opc.tcp://") {
                            self.server_input.clone()
                        } else if self.server_input.contains(':') {
                            format!("opc.tcp://{}", self.server_input)
                        } else {
                            format!("opc.tcp://{}:4840", self.server_input)
                        }
                    });

                action = Some(ConnectionAction::Connect(ClientConfig {
                    endpoint_url,
                    security_policy: self.security_policy.clone(),
                    security_mode: self.security_mode.clone(),
                    auth_method,
                }));
            }

            if ui.add_enabled(interactive, egui::Button::new(format!("💾 {}", i18n::t(T::SaveBookmark, lang))))
                .on_hover_text("Save this server configuration")
                .clicked() 
            {
                self.show_add_bookmark = true;
                self.bookmark_name = format!("Server {}", bookmarks.servers.len() + 1);
            }
        });

        
        if self.show_add_bookmark {
            egui::Window::new(i18n::t(T::SaveBookmark, lang))
                .collapsible(false)
                .resizable(false)
                .show(ui.ctx(), |ui| {
                    ui.horizontal(|ui| {
                        ui.label(i18n::t(T::Name, lang));
                        ui.text_edit_singleline(&mut self.bookmark_name);
                    });
                    
                    ui.horizontal(|ui| {
                        if ui.button(i18n::t(T::Save, lang)).clicked() {
                            let auth_method = if self.use_auth {
                                AuthMethod::UserPassword {
                                    username: self.username.clone(),
                                    password: self.password.clone(),
                                }
                            } else {
                                AuthMethod::Anonymous
                            };

                            let endpoint_url = if self.server_input.starts_with("opc.tcp://") {
                                self.server_input.clone()
                            } else {
                                format!("opc.tcp://{}", self.server_input)
                            };

                            let bookmark = ServerBookmark {
                                name: self.bookmark_name.clone(),
                                endpoint_url,
                                security_policy: self.security_policy.clone(),
                                security_mode: self.security_mode.clone(),
                                auth_method,
                            };

                            bookmarks.add(bookmark);
                            let _ = bookmarks.save();
                            self.show_add_bookmark = false;
                        }
                        
                        if ui.button(i18n::t(T::Cancel, lang)).clicked() {
                            self.show_add_bookmark = false;
                        }
                    });
                });
        }

        action
    }
}
