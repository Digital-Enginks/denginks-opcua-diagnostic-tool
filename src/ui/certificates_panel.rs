



use eframe::egui;
use std::path::PathBuf;
use crate::opcua::certificates::{CertificateManager, CertificateInfo};
use crate::utils::i18n::{self, T, Language};


#[derive(Debug)]
pub enum CertAction {
    
    TrustCert(PathBuf),
    
    DeleteCert(PathBuf),
    
    OpenPkiFolder,
    
    Refresh,
}


pub struct CertificatesPanel {
    
    cert_manager: CertificateManager,
    
    client_cert: Option<CertificateInfo>,
    
    trusted_certs: Vec<CertificateInfo>,
    
    rejected_certs: Vec<CertificateInfo>,
    
    status: String,
    
    needs_refresh: bool,
}

impl Default for CertificatesPanel {
    fn default() -> Self {
        let cert_manager = CertificateManager::default();
        
        let _ = cert_manager.ensure_pki_structure();
        
        let mut panel = Self {
            cert_manager,
            client_cert: None,
            trusted_certs: Vec::new(),
            rejected_certs: Vec::new(),
            status: String::new(),
            needs_refresh: true,
        };
        panel.refresh();
        panel
    }
}

impl CertificatesPanel {
    
    pub fn refresh(&mut self) {
        self.client_cert = self.cert_manager.get_client_cert();
        self.trusted_certs = self.cert_manager.list_trusted_certs();
        self.rejected_certs = self.cert_manager.list_rejected_certs();
        self.needs_refresh = false;
    }

    
    pub fn handle_action(&mut self, action: &CertAction) {
        match action {
            CertAction::TrustCert(path) => {
                match self.cert_manager.trust_certificate(path) {
                    Ok(()) => {
                        self.status = "✅ Certificate trusted".to_string();
                        self.needs_refresh = true;
                    }
                    Err(e) => {
                        self.status = format!("❌ Error: {}", e);
                    }
                }
            }
            CertAction::DeleteCert(path) => {
                match self.cert_manager.delete_certificate(path) {
                    Ok(()) => {
                        self.status = "✅ Certificate deleted".to_string();
                        self.needs_refresh = true;
                    }
                    Err(e) => {
                        self.status = format!("❌ Error: {}", e);
                    }
                }
            }
            CertAction::OpenPkiFolder => {
                if let Err(e) = self.cert_manager.open_pki_folder() {
                    self.status = format!("❌ Error: {}", e);
                }
            }
            CertAction::Refresh => {
                self.needs_refresh = true;
            }
        }

        if self.needs_refresh {
            self.refresh();
        }
    }

    
    pub fn show(&mut self, ui: &mut egui::Ui, lang: Language) -> Option<CertAction> {
        let mut action = None;

        ui.heading(format!("🔐 {}", i18n::t(T::Certificates, lang)));
        ui.separator();

        
        ui.horizontal(|ui| {
            if ui.button(format!("📂 {}", i18n::t(T::OpenPkiFolder, lang))).clicked() {
                action = Some(CertAction::OpenPkiFolder);
            }
            if ui.button("🔄").on_hover_text("Refresh").clicked() {
                action = Some(CertAction::Refresh);
            }
        });

        if !self.status.is_empty() {
            ui.label(&self.status);
        }

        ui.add_space(10.0);

        
        egui::CollapsingHeader::new(format!("📄 {}", i18n::t(T::ClientCertificate, lang)))
            .default_open(true)
            .show(ui, |ui| {
                if let Some(cert) = &self.client_cert {
                    ui.horizontal(|ui| {
                        ui.label("📜");
                        ui.label(&cert.name);
                    });
                    ui.label(egui::RichText::new(cert.path.display().to_string()).small().weak());
                } else {
                    ui.colored_label(
                        egui::Color32::from_rgb(150, 150, 150),
                        i18n::t(T::NoCertificates, lang)
                    );
                    ui.label(egui::RichText::new("A client certificate will be generated on first secure connection.").small().weak());
                }
            });

        ui.add_space(5.0);

        
        egui::CollapsingHeader::new(format!("✅ {} ({})", i18n::t(T::TrustedCerts, lang), self.trusted_certs.len()))
            .default_open(true)
            .show(ui, |ui| {
                if self.trusted_certs.is_empty() {
                    ui.colored_label(
                        egui::Color32::from_rgb(150, 150, 150),
                        i18n::t(T::NoCertificates, lang)
                    );
                } else {
                    egui::ScrollArea::vertical().max_height(150.0).show(ui, |ui| {
                        let mut cert_to_delete = None;
                        for cert in &self.trusted_certs {
                            ui.horizontal(|ui| {
                                ui.label("📜");
                                ui.label(&cert.name);
                                if ui.small_button("🗑").on_hover_text(i18n::t(T::DeleteCert, lang)).clicked() {
                                    cert_to_delete = Some(cert.path.clone());
                                }
                            });
                        }
                        if let Some(path) = cert_to_delete {
                            action = Some(CertAction::DeleteCert(path));
                        }
                    });
                }
            });

        ui.add_space(5.0);

        
        egui::CollapsingHeader::new(format!("❌ {} ({})", i18n::t(T::RejectedCerts, lang), self.rejected_certs.len()))
            .default_open(true)
            .show(ui, |ui| {
                if self.rejected_certs.is_empty() {
                    ui.colored_label(
                        egui::Color32::from_rgb(150, 150, 150),
                        i18n::t(T::NoCertificates, lang)
                    );
                } else {
                    ui.label(egui::RichText::new("These certificates were rejected. Trust them to allow connections.").small().weak());
                    egui::ScrollArea::vertical().max_height(150.0).show(ui, |ui| {
                        let mut cert_action_req = None;
                        for cert in &self.rejected_certs {
                            ui.horizontal(|ui| {
                                ui.label("📜");
                                ui.label(&cert.name);
                                if ui.small_button("✅").on_hover_text(i18n::t(T::TrustCert, lang)).clicked() {
                                    cert_action_req = Some(CertAction::TrustCert(cert.path.clone()));
                                }
                                if ui.small_button("🗑").on_hover_text(i18n::t(T::DeleteCert, lang)).clicked() {
                                    cert_action_req = Some(CertAction::DeleteCert(cert.path.clone()));
                                }
                            });
                        }
                        if let Some(a) = cert_action_req {
                            action = Some(a);
                        }
                    });
                }
            });

        action
    }
}
