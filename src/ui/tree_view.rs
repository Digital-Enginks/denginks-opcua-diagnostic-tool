use eframe::egui;
use opcua::types::NodeId;
use std::collections::HashMap;
use std::cell::RefCell;

use crate::opcua::browser::BrowsedNode;
use crate::utils::i18n::{self, T, Language};
use crate::opcua::browser::NodeClass;


pub enum TreeViewAction {
    Select(BrowsedNode),
    Expand(NodeId),
    ExportJson(BrowsedNode),
    ExportCsv(BrowsedNode),
    AddToWatchlist(BrowsedNode),
}


pub struct TreeView<'a> {
    /// Cache of loaded child nodes
    node_cache: &'a HashMap<NodeId, Vec<BrowsedNode>>,
    
    selected_node_id: &'a Option<NodeId>,
}

impl<'a> TreeView<'a> {
    pub fn new(
        node_cache: &'a HashMap<NodeId, Vec<BrowsedNode>>,
        selected_node_id: &'a Option<NodeId>,
    ) -> Self {
        Self {
            node_cache,
            selected_node_id,
        }
    }

    
    
    
    
    pub fn show(
        &self,
        ui: &mut egui::Ui,
        nodes: &[BrowsedNode],
        lang: Language,
    ) -> Vec<TreeViewAction> {
        let mut actions = Vec::new();

        for node in nodes {
            actions.extend(self.show_node(ui, node, lang));
        }

        actions
    }

    fn show_node(
        &self,
        ui: &mut egui::Ui,
        node: &BrowsedNode,
        lang: Language,
    ) -> Vec<TreeViewAction> {
        let actions = RefCell::new(Vec::new());

        
        let icon = node.node_class.icon();
        let text = format!("{} {}", icon, node.display_name);
        
        
        let id = ui.make_persistent_id(node.node_id.to_string());
        let is_selected = self.selected_node_id.as_ref() == Some(&node.node_id);

        
        let context_menu = |ui: &mut egui::Ui| {
            if node.has_children || node.node_class == NodeClass::Object {
                ui.label(i18n::t(T::Actions, lang));
                ui.separator();
                if ui.button(format!("💾 {}", i18n::t(T::ExportJSON, lang))).clicked() {
                    actions.borrow_mut().push(TreeViewAction::ExportJson(node.clone()));
                    ui.close_menu();
                }
                if ui.button(format!("💾 {}", i18n::t(T::ExportCSV, lang))).clicked() {
                    actions.borrow_mut().push(TreeViewAction::ExportCsv(node.clone()));
                    ui.close_menu();
                }
            }
            
            if node.node_class == NodeClass::Variable {
                ui.label(i18n::t(T::Actions, lang));
                ui.separator();
                
                 if ui.button(format!("📊 {}", i18n::t(T::Watchlist, lang))).clicked() {
                    actions.borrow_mut().push(TreeViewAction::AddToWatchlist(node.clone()));
                    ui.close_menu();
                }
            }
        };

        
        if node.has_children {
            let state = egui::collapsing_header::CollapsingState::load_with_default_open(
                ui.ctx(),
                id,
                false,
            );

            let header_response = state.show_header(ui, |ui| {
                let response = ui.selectable_label(is_selected, text);
                if response.clicked() {
                     actions.borrow_mut().push(TreeViewAction::Select(node.clone()));
                }
                response.context_menu(context_menu);
            });
            
            header_response.body(|ui| {
                if let Some(children) = self.node_cache.get(&node.node_id) {
                    actions.borrow_mut().extend(self.show(ui, children, lang));
                } else {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label(i18n::t(T::Checking, lang)); 
                    });
                     actions.borrow_mut().push(TreeViewAction::Expand(node.node_id.clone()));
                }
            });

        } else {
            let response = ui.selectable_label(is_selected, text);
            if response.clicked() {
                 actions.borrow_mut().push(TreeViewAction::Select(node.clone()));
            }
            
            if response.double_clicked() && node.node_class == NodeClass::Variable {
                actions.borrow_mut().push(TreeViewAction::AddToWatchlist(node.clone()));
            }
            response.context_menu(context_menu);
        }

        actions.into_inner()
    }
}
