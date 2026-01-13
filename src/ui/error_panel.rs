



use eframe::egui;
use std::collections::VecDeque;
use std::time::Instant;

use crate::utils::i18n::{self, T, Language};


const MAX_NOTIFICATIONS: usize = 10;


const TOAST_DURATION_SECS: u64 = 5;


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ErrorSeverity {
    #[allow(dead_code)]
    Info,
    Warning,
    Error,
}

impl ErrorSeverity {
    pub fn icon(&self) -> &'static str {
        match self {
            ErrorSeverity::Info => "ℹ️",
            ErrorSeverity::Warning => "⚠️",
            ErrorSeverity::Error => "❌",
        }
    }

    pub fn color(&self) -> egui::Color32 {
        match self {
            ErrorSeverity::Info => egui::Color32::from_rgb(100, 180, 255),
            ErrorSeverity::Warning => egui::Color32::from_rgb(255, 200, 50),
            ErrorSeverity::Error => egui::Color32::from_rgb(255, 80, 80),
        }
    }
}

/// An error notification
#[derive(Debug, Clone)]
pub struct ErrorNotification {
    pub message: String,
    pub severity: ErrorSeverity,
    pub timestamp: Instant,
    pub details: Option<String>,
}

impl ErrorNotification {
    pub fn new(message: impl Into<String>, severity: ErrorSeverity) -> Self {
        Self {
            message: message.into(),
            severity,
            timestamp: Instant::now(),
            details: None,
        }
    }

    #[allow(dead_code)]
    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }

    /// Check if this notification should still be shown as a toast
    pub fn is_toast_active(&self) -> bool {
        self.timestamp.elapsed().as_secs() < TOAST_DURATION_SECS
    }
}

/// Common OPC-UA error codes and their descriptions
pub fn get_common_errors(lang: Language) -> Vec<(&'static str, &'static str, &'static str)> {
    match lang {
        Language::English => vec![
            ("BadCertificateInvalid", "Certificate is invalid", "The server rejected your client certificate. Try regenerating it."),
            ("BadCertificateHostNameInvalid", "Certificate hostname mismatch", "The certificate hostname doesn't match the server. Check your endpoint URL."),
            ("BadCertificateUntrusted", "Certificate not trusted", "The server certificate is not trusted. Add it to your trusted certificates."),
            ("BadSecurityModeRejected", "Security mode rejected", "The server doesn't support this security mode. Try a different security policy."),
            ("BadIdentityTokenRejected", "Authentication failed", "Username/password rejected. Check your credentials."),
            ("BadUserAccessDenied", "Access denied", "Your user account doesn't have permission to access this resource."),
            ("BadConnectionClosed", "Connection closed", "The server closed the connection. It may have restarted or timed out."),
            ("BadTimeout", "Timeout", "The operation took too long. Check network connectivity."),
            ("BadNotConnected", "Not connected", "No active connection to the server."),
            ("BadServiceUnsupported", "Service not supported", "The server doesn't support this operation."),
        ],
        Language::Spanish => vec![
            ("BadCertificateInvalid", "Certificado inválido", "El servidor rechazó tu certificado de cliente. Intenta regenerarlo."),
            ("BadCertificateHostNameInvalid", "Nombre de host no coincide", "El nombre de host del certificado no coincide con el servidor."),
            ("BadCertificateUntrusted", "Certificado no confiable", "El certificado del servidor no es confiable. Agrégalo a certificados confiables."),
            ("BadSecurityModeRejected", "Modo de seguridad rechazado", "El servidor no soporta este modo de seguridad. Prueba otra política."),
            ("BadIdentityTokenRejected", "Autenticación fallida", "Usuario/contraseña rechazados. Verifica tus credenciales."),
            ("BadUserAccessDenied", "Acceso denegado", "Tu cuenta no tiene permiso para acceder a este recurso."),
            ("BadConnectionClosed", "Conexión cerrada", "El servidor cerró la conexión. Puede haberse reiniciado."),
            ("BadTimeout", "Tiempo agotado", "La operación tardó demasiado. Verifica la conectividad de red."),
            ("BadNotConnected", "No conectado", "No hay conexión activa al servidor."),
            ("BadServiceUnsupported", "Servicio no soportado", "El servidor no soporta esta operación."),
        ],
    }
}


#[derive(Default)]
pub struct ErrorPanel {
    
    pub notifications: VecDeque<ErrorNotification>,
    
    #[allow(dead_code)]
    pub show_panel: bool,
    
    pub show_reference: bool,
}

impl ErrorPanel {
    
    pub fn add_error(&mut self, message: impl Into<String>, severity: ErrorSeverity) {
        let notification = ErrorNotification::new(message, severity);
        self.notifications.push_front(notification);
        
        
        while self.notifications.len() > MAX_NOTIFICATIONS {
            self.notifications.pop_back();
        }
    }

    
    #[allow(dead_code)]
    pub fn add_error_with_details(&mut self, message: impl Into<String>, details: impl Into<String>, severity: ErrorSeverity) {
        let notification = ErrorNotification::new(message, severity).with_details(details);
        self.notifications.push_front(notification);
        
        while self.notifications.len() > MAX_NOTIFICATIONS {
            self.notifications.pop_back();
        }
    }

    
    pub fn clear(&mut self) {
        self.notifications.clear();
    }

    
    #[allow(dead_code)]
    pub fn has_active_toasts(&self) -> bool {
        self.notifications.iter().any(|n| n.is_toast_active())
    }

    
    pub fn show_toasts(&self, ctx: &egui::Context) {
        let active_toasts: Vec<_> = self.notifications.iter()
            .filter(|n| n.is_toast_active())
            .take(3) 
            .collect();

        if active_toasts.is_empty() {
            return;
        }

        
        egui::Area::new(egui::Id::new("error_toasts"))
            .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-10.0, 40.0))
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    for toast in active_toasts {
                        let elapsed = toast.timestamp.elapsed().as_secs_f32();
                        let alpha = if elapsed > (TOAST_DURATION_SECS as f32 - 1.0) {
                            1.0 - (elapsed - (TOAST_DURATION_SECS as f32 - 1.0))
                        } else {
                            1.0
                        }.clamp(0.0, 1.0);

                        let frame_color = toast.severity.color().gamma_multiply(alpha);
                        
                        ui.group(|ui| {
                            ui.visuals_mut().widgets.noninteractive.bg_fill = 
                                egui::Color32::from_rgba_unmultiplied(40, 40, 40, (220.0 * alpha) as u8);
                            ui.visuals_mut().widgets.noninteractive.bg_stroke = 
                                egui::Stroke::new(2.0, frame_color);
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new(toast.severity.icon()).size(16.0));
                                ui.label(egui::RichText::new(&toast.message).color(egui::Color32::WHITE));
                            });
                        });
                        ui.add_space(5.0);
                    }
                });
            });
    }

    
    pub fn show_panel(&mut self, ui: &mut egui::Ui, lang: Language) {
        ui.heading(format!("{} {}", "⚠️", i18n::t(T::ErrorPanel, lang)));
        
        ui.horizontal(|ui| {
            if ui.button(i18n::t(T::ClearAll, lang)).clicked() {
                self.clear();
            }
            ui.checkbox(&mut self.show_reference, i18n::t(T::CommonErrors, lang));
        });
        
        ui.separator();

        if self.show_reference {
            ui.collapsing(i18n::t(T::CommonErrors, lang), |ui| {
                egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                    egui::Grid::new("common_errors_grid").striped(true).show(ui, |ui| {
                        ui.strong(i18n::t(T::ErrorCode, lang));
                        ui.strong(i18n::t(T::ErrorDescription, lang));
                        ui.end_row();

                        for (code, desc, solution) in get_common_errors(lang) {
                            ui.label(egui::RichText::new(code).monospace());
                            ui.label(desc).on_hover_text(solution);
                            ui.end_row();
                        }
                    });
                });
            });
            ui.separator();
        }

        if self.notifications.is_empty() {
            ui.label(i18n::t(T::NoErrors, lang));
        } else {
            egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                for notification in &self.notifications {
                    let elapsed = notification.timestamp.elapsed();
                    let time_str = if elapsed.as_secs() < 60 {
                        format!("{}s ago", elapsed.as_secs())
                    } else {
                        format!("{}m ago", elapsed.as_secs() / 60)
                    };

                    ui.group(|ui| {
                        ui.visuals_mut().widgets.noninteractive.bg_fill = egui::Color32::from_rgb(35, 35, 35);
                        ui.visuals_mut().widgets.noninteractive.bg_stroke = 
                            egui::Stroke::new(1.0, notification.severity.color());
                        ui.horizontal(|ui| {
                            ui.label(notification.severity.icon());
                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    ui.strong(&notification.message);
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        ui.label(egui::RichText::new(&time_str).small().weak());
                                    });
                                });
                                if let Some(details) = &notification.details {
                                    ui.label(egui::RichText::new(details).small().weak());
                                }
                            });
                        });
                    });
                    ui.add_space(4.0);
                }
            });
        }
    }
}
