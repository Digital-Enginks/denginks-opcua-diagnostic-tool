use eframe::egui;
use crate::opcua::browser::{BrowsedNode, NodeClass};
use crate::utils::i18n::{self, T, Language};

/// Actions triggered from properties panel
pub enum PropertiesAction {
    AddToWatchlist(BrowsedNode),
}

/// Properties panel component
pub struct PropertiesPanel<'a> {
    selected_node: &'a Option<BrowsedNode>,
    monitored_data: Option<&'a crate::opcua::subscription::MonitoredData>,
}

impl<'a> PropertiesPanel<'a> {
    pub fn new(
        selected_node: &'a Option<BrowsedNode>,
        monitored_data: Option<&'a crate::opcua::subscription::MonitoredData>,
    ) -> Self {
        Self { selected_node, monitored_data }
    }

    pub fn show(&self, ui: &mut egui::Ui, lang: Language) -> Option<PropertiesAction> {
        let mut action = None;
        ui.heading(i18n::t(T::Properties, lang));
        ui.separator();

        if let Some(node) = self.selected_node {
            egui::Grid::new("properties_grid")
                .num_columns(2)
                .spacing([10.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    // Basic Attributes
                    ui.label(format!("{} ", i18n::t(T::DisplayName, lang)));
                    ui.label(&node.display_name);
                    ui.end_row();

                    ui.label("Browse Name:");
                    ui.label(&node.browse_name);
                    ui.end_row();

                    ui.label(format!("{} ", i18n::t(T::NodeId, lang)));
                    ui.horizontal(|ui| {
                        ui.label(node.node_id.to_string());
                        if ui.button("📋").on_hover_text("Copy Node ID").clicked() {
                            ui.ctx().copy_text(node.node_id.to_string());
                        }
                    });
                    ui.end_row();

                    ui.label("Node Class:");
                    ui.horizontal(|ui| {
                        ui.label(node.node_class.icon());
                        ui.label(node.node_class.to_string());
                    });
                    ui.end_row();

                    if let Some(type_def) = &node.type_definition {
                        ui.label("Type Def:");
                        ui.label(type_def.to_string());
                        ui.end_row();
                    }

                    // Live Data (if monitored)
                    if let Some(data) = self.monitored_data {
                        ui.label(format!("{} ", i18n::t(T::Value, lang)));
                        ui.label(egui::RichText::new(data.value_string()).strong());
                        ui.end_row();

                        ui.label(format!("{} ", i18n::t(T::Timestamp, lang)));
                        ui.label(data.timestamp_string());
                        ui.end_row();
                    }
                });

            ui.add_space(20.0);
            
            // Actions for Variables
            if node.node_class == NodeClass::Variable {
                ui.separator();
                ui.heading(i18n::t(T::Actions, lang));
                ui.horizontal(|ui| {
                    if ui.button(format!("📊 {}", i18n::t(T::Watchlist, lang))).on_hover_text("Monitor this value in real-time").clicked() {
                        action = Some(PropertiesAction::AddToWatchlist(node.clone()));
                    }
                });
            }

        } else {
            ui.label("Select a node to view properties.");
        }
        
        action
    }
}
