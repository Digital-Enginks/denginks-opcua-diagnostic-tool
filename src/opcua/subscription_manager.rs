//! Subscription Manager
//!
//! Handles the lifecycle of OPC-UA subscriptions and monitored items.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::runtime::Handle;
use tokio::sync::RwLock;

use opcua::types::{NodeId, StatusCode};
use crate::opcua::client::OpcUaClient;
use crate::opcua::subscription::{MonitoredData, SubscriptionState};
use crate::app::BackendMessage;
use crate::opcua::browser::BrowsedNode;

/// Action required after adding a node to watchlist
pub enum SubscriptionAction {
    /// No action needed (wait for subscription or already added)
    None,
    /// Need to start a subscription creation task
    CreateSubscription,
    /// Need to add items to existing subscription
    AddItems(Vec<NodeId>),
}

/// Manages subscriptions and monitored items
#[derive(Default)]
pub struct SubscriptionManager {
    /// Live monitored items [NodeId -> Data]
    pub monitored_items: HashMap<NodeId, MonitoredData>,
    
    /// Subscription state tracker
    pub subscription_state: SubscriptionState,
    
    /// Items waiting for subscription creation
    pub pending_monitored_items: Vec<NodeId>,
    
    /// Is a subscription currently being created?
    pub creating_subscription: bool,
}

impl SubscriptionManager {
    /// Create a new manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Clear all state (e.g. on disconnect)
    pub fn clear(&mut self) {
        self.monitored_items.clear();
        self.subscription_state.clear();
        self.pending_monitored_items.clear();
        self.creating_subscription = false;
    }

    /// Add a node to the watchlist
    pub fn request_add_to_watchlist(&mut self, node: &BrowsedNode) -> SubscriptionAction {
        if self.monitored_items.contains_key(&node.node_id) {
            return SubscriptionAction::None;
        }

        // Create initial data entry
        let data = MonitoredData::new(node.node_id.clone(), node.display_name.clone());
        self.monitored_items.insert(node.node_id.clone(), data);

        // Handle subscription
        if self.subscription_state.subscription_id.is_some() {
             SubscriptionAction::AddItems(vec![node.node_id.clone()])
        } else {
             // Add to pending
             self.pending_monitored_items.push(node.node_id.clone());
             
             // Create subscription if not already creating
             if !self.creating_subscription {
                 self.creating_subscription = true;
                 SubscriptionAction::CreateSubscription
             } else {
                 SubscriptionAction::None
             }
        }
    }
    
    pub fn spawn_subscription_task(
        &self,
        runtime: &Handle,
        opcua_client: Arc<RwLock<Option<OpcUaClient>>>,
        backend_tx: std::sync::mpsc::Sender<BackendMessage>,
    ) {
        let tx = backend_tx;
        let client_handle = opcua_client;
        
        runtime.spawn(async move {
            let guard = client_handle.read().await;
            if let Some(client) = guard.as_ref() {
                // Define the callback
                let tx_cb = tx.clone();
                let callback = move |data_value: opcua::types::DataValue, item: &opcua::client::MonitoredItem| {
                    let item_id = item.client_handle();
                    let _ = tx_cb.send(BackendMessage::DataChange(item_id, data_value));
                };

                match client.create_subscription(std::time::Duration::from_millis(500), callback).await {
                    Ok(id) => {
                        let _ = tx.send(BackendMessage::SubscriptionCreated(id));
                    }
                    Err(e) => {
                        let _ = tx.send(BackendMessage::Error(format!("Failed to create subscription: {}", e)));
                    }
                }
            }
        });
    }

    pub fn spawn_add_items_task(
        &mut self,
        runtime: &Handle,
        opcua_client: Arc<RwLock<Option<OpcUaClient>>>,
        backend_tx: std::sync::mpsc::Sender<BackendMessage>,
    ) {
        let sub_id = self.subscription_state.subscription_id.unwrap_or(0);
        if sub_id == 0 { return; }
        
        // Take pending items
        if self.pending_monitored_items.is_empty() { return; }
        let node_ids = std::mem::take(&mut self.pending_monitored_items);
        
        let tx = backend_tx;
        let client_handle = opcua_client;
        
        runtime.spawn(async move {
            let guard = client_handle.read().await;
            if let Some(client) = guard.as_ref() {
                match client.add_monitored_items(sub_id, &node_ids).await {
                    Ok(pairs) => {
                         let _ = tx.send(BackendMessage::MonitoredItemsAdded(pairs));
                    }
                    Err(e) => {
                        let _ = tx.send(BackendMessage::Error(format!("Failed to add items: {}", e)));
                    }
                }
            }
        });
    }
    
    pub fn spawn_add_specific_items_task(
        &self,
        node_ids: Vec<NodeId>,
        runtime: &Handle,
        opcua_client: Arc<RwLock<Option<OpcUaClient>>>,
        backend_tx: std::sync::mpsc::Sender<BackendMessage>,
    ) {
         let sub_id = self.subscription_state.subscription_id.unwrap_or(0);
         if sub_id == 0 { return; }
         
         let tx = backend_tx;
         let client_handle = opcua_client;

         runtime.spawn(async move {
            let guard = client_handle.read().await;
            if let Some(client) = guard.as_ref() {
                match client.add_monitored_items(sub_id, &node_ids).await {
                    Ok(pairs) => {
                         let _ = tx.send(BackendMessage::MonitoredItemsAdded(pairs));
                    }
                    Err(e) => {
                        let _ = tx.send(BackendMessage::Error(format!("Failed to add items: {}", e)));
                    }
                }
            }
        });
    }

    pub fn remove_from_watchlist(
        &mut self,
        node_id: &NodeId,
        runtime: &Handle,
        opcua_client: Arc<RwLock<Option<OpcUaClient>>>,
    ) {
        if let Some(item_id) = self.subscription_state.unregister_by_node(node_id) {
             if let Some(sub_id) = self.subscription_state.subscription_id {
                 self.spawn_remove_items_task(sub_id, vec![item_id], runtime, opcua_client);
             }
        }
        self.monitored_items.remove(node_id);
    }
    
    fn spawn_remove_items_task(
        &self,
        sub_id: u32,
        item_ids: Vec<u32>,
        runtime: &Handle,
        opcua_client: Arc<RwLock<Option<OpcUaClient>>>,
    ) {
        let client_handle = opcua_client;
        runtime.spawn(async move {
            let guard = client_handle.read().await;
             if let Some(client) = guard.as_ref() {
                 let _ = client.remove_monitored_items(sub_id, &item_ids).await;
             }
        });
    }
    
    pub fn handle_data_change(&mut self, handle: u32, value: opcua::types::DataValue) {
        if let Some(node_id) = self.subscription_state.get_node_id(handle) {
             if let Some(item) = self.monitored_items.get_mut(node_id) {
                item.update(&value);
            }
        }
    }
    
    pub fn handle_monitored_items_added(&mut self, pairs: Vec<(NodeId, u32, u32)>) {
         for (node_id, item_id, handle) in pairs {
            self.subscription_state.register_item(node_id.clone(), item_id, handle);
            if let Some(item) = self.monitored_items.get_mut(&node_id) {
                item.monitored_item_id = Some(item_id);
                item.status = StatusCode::Good; 
            }
        }
    }
}
