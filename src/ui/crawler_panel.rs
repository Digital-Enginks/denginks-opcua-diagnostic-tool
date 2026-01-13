//! Crawler Panel UI
//!
//! Provides configuration and results view for the network crawler.

use eframe::egui;
use opcua::types::NodeId;
use crate::opcua::browser::BrowsedNode;
use crate::opcua::crawler::CrawlConfig;
use crate::utils::i18n::{self, T, Language};

/// Actions from the crawler panel
pub enum CrawlerAction {
    StartCrawl(CrawlConfig),
    ExportJson,
    ExportCsv,
    #[allow(dead_code)]
    JumpToNode(NodeId),
}

/// State for the crawler panel
pub struct CrawlerPanel {
    /// Configuration
    pub config: CrawlConfig,
    /// Results of the last crawl
    pub results: Vec<BrowsedNode>,
    /// Is a crawl in progress?
    pub is_crawling: bool,
    /// Status message
    pub status: String,
    /// Start time of the crawl
    pub start_time: Option<std::time::Instant>,
}

impl Default for CrawlerPanel {
    fn default() -> Self {
        Self {
            config: CrawlConfig {
                max_depth: 5,
                max_nodes: 500_000, // Allow large values internally
                start_node: NodeId::from(opcua::types::ObjectId::RootFolder),
            },
            results: Vec::new(),
            is_crawling: false,
            status: String::new(),
            start_time: None,
        }
    }
}

impl CrawlerPanel {
    /// Show the panel
    pub fn show(&mut self, ui: &mut egui::Ui, is_connected: bool, lang: Language) -> Option<CrawlerAction> {
        let mut action = None;

        ui.heading(format!("🕷 {}", i18n::t(T::Crawler, lang)));
        ui.label(i18n::t(T::CrawlerDescription, lang));
        ui.separator();

        if !is_connected {
            ui.colored_label(egui::Color32::from_rgb(255, 100, 100), i18n::t(T::ConnectToUseCrawler, lang));
            return None;
        }

        // Configuration - simplified (only depth and start node)
        ui.group(|ui| {
            ui.label(i18n::t(T::Configuration, lang));
            ui.horizontal(|ui| {
                ui.label(format!("{} ", i18n::t(T::Node, lang)));
                ui.label(self.config.start_node.to_string());
            });

            ui.add(egui::Slider::new(&mut self.config.max_depth, 1..=10).text(i18n::t(T::MaxDepth, lang)));
            // Max nodes slider removed - uses internal default of 500k
        });

        ui.add_space(5.0);

        // Actions
        ui.horizontal(|ui| {
            if self.is_crawling {
                ui.add(egui::Spinner::new());
                if let Some(start) = self.start_time {
                     let elapsed = start.elapsed().as_secs();
                     ui.label(format!("{} ({}s)", i18n::t(T::Checking, lang), elapsed));
                } else {
                     ui.label(i18n::t(T::Checking, lang));
                }
            } else if ui.button(format!("▶ {}", i18n::t(T::StartCrawl, lang))).clicked() {
                action = Some(CrawlerAction::StartCrawl(self.config.clone()));
                self.is_crawling = true;
                self.results.clear();
                self.status = i18n::t(T::Connecting, lang).to_string(); 
                self.start_time = Some(std::time::Instant::now());
            }
        });

        ui.separator();

        // Results - simplified message and export buttons only
        if !self.results.is_empty() {
            ui.vertical(|ui| {
                ui.colored_label(
                    egui::Color32::from_rgb(100, 200, 100),
                    format!("✓ {} {} {}", i18n::t(T::CrawlComplete, lang).split('.').next().unwrap_or("Complete"), self.results.len(), "nodes")
                );
                
                ui.add_space(10.0);
                
                ui.horizontal(|ui| {
                    if ui.button(format!("💾 {}", i18n::t(T::ExportJSON, lang))).clicked() {
                        action = Some(CrawlerAction::ExportJson);
                    }
                    if ui.button(format!("📄 {}", i18n::t(T::ExportCSV, lang))).clicked() {
                        action = Some(CrawlerAction::ExportCsv);
                    }
                });
            });
        } else if !self.status.is_empty() {
            ui.label(&self.status);
        }

        action
    }
}
