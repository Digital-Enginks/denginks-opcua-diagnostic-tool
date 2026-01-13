




use eframe::egui;
use egui_extras::{Column, TableBuilder};
use opcua::types::NodeId;
use std::collections::HashMap;

use crate::opcua::subscription::MonitoredData;
use crate::utils::i18n::{self, T, Language};
use crate::ui::trending::color_for_node_id;


pub enum MonitorAction {
    
    Remove(NodeId),
    
    ToggleTrend(NodeId),
    
    ChangeColor(NodeId, [u8; 3]),
    
    ExportCsv,
    
    ExportJson,
}



#[derive(Default)]
pub struct MonitorPanel;

impl MonitorPanel {
    
    pub fn show(
        &self,
        ui: &mut egui::Ui,
        monitored_items: &HashMap<NodeId, MonitoredData>,
        lang: Language,
    ) -> Option<MonitorAction> {
        let mut action: Option<MonitorAction> = None;

        ui.heading(format!("📊 {}", i18n::t(T::Watchlist, lang)));
        ui.horizontal(|ui| {
             if ui.button(format!("💾 {}", i18n::t(T::ExportCSV, lang))).clicked() {
                 action = Some(MonitorAction::ExportCsv);
             }
             if ui.button(format!("💾 {}", i18n::t(T::ExportJSON, lang))).clicked() {
                 action = Some(MonitorAction::ExportJson);
             }
        });
        ui.separator();

        
        if monitored_items.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label(i18n::t(T::NoItems, lang));
            });
            return None;
        }

        
        TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(Column::auto().resizable(true)) 
            .column(Column::remainder())            
            .column(Column::auto())                 
            .column(Column::auto())                 
            .column(Column::auto())                 
            .header(20.0, |mut header| {
                header.col(|ui| { ui.strong(i18n::t(T::Node, lang)); });
                header.col(|ui| { ui.strong(i18n::t(T::Value, lang)); });
                header.col(|ui| { ui.strong(i18n::t(T::Quality, lang)); });
                header.col(|ui| { ui.strong(i18n::t(T::Timestamp, lang)); });
                header.col(|ui| { ui.strong(i18n::t(T::Actions, lang)); });
            })
            .body(|mut body| {
                
                let mut keys: Vec<&NodeId> = monitored_items.keys().collect();
                keys.sort_by_key(|k| &monitored_items[k].display_name);

                for node_id in keys {
                    let item = &monitored_items[node_id];
                    let is_trendable = item.is_trendable();
                    
                    body.row(20.0, |mut row| {
                        
                        row.col(|ui| {
                            ui.label(&item.display_name).on_hover_text(node_id.to_string());
                        });

                        
                        row.col(|ui| {
                            ui.label(item.value_string());
                        });

                        
                        row.col(|ui| {
                            let (text, color) = match item.quality_icon() {
                                "OK" => ("OK", egui::Color32::GREEN),
                                "?" => ("?", egui::Color32::from_rgb(255, 165, 0)), 
                                _ => ("!", egui::Color32::RED),
                            };
                            ui.colored_label(color, text)
                                .on_hover_text(crate::opcua::status_codes::translate_status_code(item.status));
                        });

                        
                        row.col(|ui| {
                            ui.label(item.timestamp_string());
                        });

                        
                        row.col(|ui| {
                            ui.horizontal(|ui| {
                                
                                if item.show_in_trend {
                                    let current_color = if let Some(rgb) = item.trend_color {
                                        egui::Color32::from_rgb(rgb[0], rgb[1], rgb[2])
                                    } else {
                                        color_for_node_id(node_id)
                                    };
                                    
                                    
                                    let (rect, response) = ui.allocate_exact_size(
                                        egui::vec2(16.0, 16.0),
                                        egui::Sense::click()
                                    );
                                    if ui.is_rect_visible(rect) {
                                        ui.painter().rect_filled(rect, 2.0, current_color);
                                        
                                        let border_rect = rect.expand(1.0);
                                        ui.painter().rect_filled(border_rect, 2.0, egui::Color32::GRAY);
                                        ui.painter().rect_filled(rect, 2.0, current_color);
                                    }
                                    
                                    
                                    response.context_menu(|ui| {
                                        ui.label("Select color:");
                                        
                                        
                                        let colors: [[u8; 3]; 12] = [
                                            [255, 0, 0],     
                                            [0, 255, 0],     
                                            [0, 0, 255],     
                                            [255, 255, 0],   
                                            [255, 0, 255],   
                                            [0, 255, 255],   
                                            [255, 128, 0],   
                                            [128, 0, 255],   
                                            [0, 255, 128],   
                                            [255, 128, 128], 
                                            [128, 255, 128], 
                                            [128, 128, 255], 
                                        ];
                                        
                                        ui.horizontal_wrapped(|ui| {
                                            for rgb in &colors {
                                                let color = egui::Color32::from_rgb(rgb[0], rgb[1], rgb[2]);
                                                if ui.add(egui::Button::new("  ").fill(color)).clicked() {
                                                    action = Some(MonitorAction::ChangeColor(node_id.clone(), *rgb));
                                                    ui.close_menu();
                                                }
                                            }
                                        });
                                    });
                                    
                                    response.on_hover_text("Right-click to change color");
                                }
                                
                                
                                if is_trendable {
                                    let trend_icon = if item.show_in_trend { "📈" } else { "📉" };
                                    let trend_tooltip = if item.show_in_trend { 
                                        "Remove from trend" 
                                    } else { 
                                        "Add to trend" 
                                    };
                                    if ui.button(trend_icon).on_hover_text(trend_tooltip).clicked() {
                                        action = Some(MonitorAction::ToggleTrend(node_id.clone()));
                                    }
                                } else {
                                    
                                    ui.add_enabled(false, egui::Button::new("📉"))
                                        .on_disabled_hover_text("Cannot graph non-numeric values (dates, strings)");
                                }

                                
                                if ui.button("🗑").on_hover_text(i18n::t(T::Remove, lang)).clicked() {
                                    action = Some(MonitorAction::Remove(node_id.clone()));
                                }
                            });
                        });
                    });
                }
            });

        action
    }
}
