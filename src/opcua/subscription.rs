//! OPC-UA Subscription management for real-time data monitoring
//!
//! Provides data structures and utilities for managing monitored items
//! and their real-time values.

use std::collections::{HashMap, VecDeque};

use opcua::types::{DataValue, NodeId, StatusCode, Variant, DateTime};

/// Maximum number of history points to keep per monitored item (for trending)
pub const MAX_HISTORY_POINTS: usize = 600;

/// Data for a single monitored item
#[derive(Debug, Clone)]
pub struct MonitoredData {
    /// The NodeId of this monitored item
    pub node_id: NodeId,
    /// Display name for UI
    pub display_name: String,
    /// The monitored item ID assigned by the server
    pub monitored_item_id: Option<u32>,
    /// Current value
    pub value: Option<Variant>,
    /// Current status code
    pub status: StatusCode,
    /// Source timestamp of the value
    pub source_timestamp: Option<DateTime>,
    /// Server timestamp of the value
    pub server_timestamp: Option<DateTime>,
    /// Historical values for trending (timestamp_seconds, numeric_value)
    pub history: VecDeque<(f64, f64)>,
    /// Whether this item is selected for trending display
    pub show_in_trend: bool,
    /// Custom color for trending (RGB) - None means auto-generate
    pub trend_color: Option<[u8; 3]>,
}

impl MonitoredData {
    /// Create a new MonitoredData instance
    pub fn new(node_id: NodeId, display_name: String) -> Self {
        Self {
            node_id,
            display_name,
            monitored_item_id: None,
            value: None,
            status: StatusCode::BadWaitingForInitialData,
            source_timestamp: None,
            server_timestamp: None,
            history: VecDeque::with_capacity(MAX_HISTORY_POINTS),
            show_in_trend: false,
            trend_color: None,
        }
    }

    /// Check if the current value is trendable (numeric)
    pub fn is_trendable(&self) -> bool {
        self.value.as_ref().and_then(variant_to_f64).is_some()
    }

    /// Update with new data value
    pub fn update(&mut self, data_value: &DataValue) {
        self.value = data_value.value.clone();
        self.status = data_value.status.unwrap_or(StatusCode::Good);
        self.source_timestamp = data_value.source_timestamp;
        self.server_timestamp = data_value.server_timestamp;

        // Add to history if the value is numeric
        if let Some(ref variant) = self.value {
            if let Some(numeric) = variant_to_f64(variant) {
                let timestamp = self.source_timestamp
                    .map(|dt| dt.as_chrono().timestamp_millis() as f64 / 1000.0)
                    .unwrap_or_else(|| {
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map(|d| d.as_secs_f64())
                            .unwrap_or(0.0)
                    });

                self.history.push_back((timestamp, numeric));

                // Trim history if too long
                while self.history.len() > MAX_HISTORY_POINTS {
                    self.history.pop_front();
                }
            }
        }
    }

    /// Get a formatted string representation of the current value
    pub fn value_string(&self) -> String {
        match &self.value {
            Some(v) => format_variant(v),
            None => "---".to_string(),
        }
    }

    /// Get the quality icon based on status code
    pub fn quality_icon(&self) -> &'static str {
        if self.status.is_good() {
            "OK"
        } else if self.status.is_uncertain() {
            "?"
        } else {
            "!"
        }
    }

    /// Get the timestamp string for display
    pub fn timestamp_string(&self) -> String {
        self.source_timestamp
            .map(|dt| {
                let chrono_dt = dt.as_chrono();
                chrono_dt.format("%d-%m-%Y %H:%M:%S").to_string()
            })
            .unwrap_or_else(|| "---".to_string())
    }
}

/// Convert a Variant to f64 for trending (returns None if not numeric)
pub fn variant_to_f64(variant: &Variant) -> Option<f64> {
    match variant {
        Variant::Boolean(b) => Some(if *b { 1.0 } else { 0.0 }),
        Variant::SByte(v) => Some(*v as f64),
        Variant::Byte(v) => Some(*v as f64),
        Variant::Int16(v) => Some(*v as f64),
        Variant::UInt16(v) => Some(*v as f64),
        Variant::Int32(v) => Some(*v as f64),
        Variant::UInt32(v) => Some(*v as f64),
        Variant::Int64(v) => Some(*v as f64),
        Variant::UInt64(v) => Some(*v as f64),
        Variant::Float(v) => Some(*v as f64),
        Variant::Double(v) => Some(*v),
        _ => None,
    }
}

/// Format a Variant for display
pub fn format_variant(variant: &Variant) -> String {
    match variant {
        Variant::Empty => "Empty".to_string(),
        Variant::Boolean(b) => b.to_string(),
        Variant::SByte(v) => v.to_string(),
        Variant::Byte(v) => v.to_string(),
        Variant::Int16(v) => v.to_string(),
        Variant::UInt16(v) => v.to_string(),
        Variant::Int32(v) => v.to_string(),
        Variant::UInt32(v) => v.to_string(),
        Variant::Int64(v) => v.to_string(),
        Variant::UInt64(v) => v.to_string(),
        Variant::Float(v) => format!("{:.4}", v),
        Variant::Double(v) => format!("{:.6}", v),
        Variant::String(s) => s.to_string(),
        Variant::DateTime(dt) => dt.as_chrono().to_rfc3339(),
        Variant::ByteString(bs) => format!("[{} bytes]", bs.len()),
        Variant::LocalizedText(lt) => lt.text.to_string(),
        Variant::QualifiedName(qn) => qn.to_string(),
        Variant::NodeId(id) => id.to_string(),
        Variant::StatusCode(sc) => format!("{:?}", sc),
        _ => format!("{:?}", variant),
    }
}

/// Subscription configuration
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SubscriptionConfig {
    /// Publishing interval in milliseconds
    pub publishing_interval_ms: u64,
    /// Lifetime count
    pub lifetime_count: u32,
    /// Max keep-alive count
    pub max_keepalive_count: u32,
    /// Max notifications per publish
    pub max_notifications: u32,
    /// Priority
    pub priority: u8,
}

impl Default for SubscriptionConfig {
    fn default() -> Self {
        Self {
            publishing_interval_ms: 1000,  // 1 second
            lifetime_count: 10,
            max_keepalive_count: 30,
            max_notifications: 0,  // unlimited
            priority: 0,
        }
    }
}

/// Message type for subscription data changes sent to UI
#[derive(Debug)]
#[allow(dead_code)]
pub struct DataChangeNotification {
    /// NodeId that changed
    pub node_id: NodeId,
    /// The new data value
    pub data_value: DataValue,
}

/// Manager for tracking subscription state
#[derive(Debug, Default)]
pub struct SubscriptionState {
    /// Active subscription ID (if any)
    pub subscription_id: Option<u32>,
    /// Mapping from ClientHandle to NodeId
    pub handle_to_node: HashMap<u32, NodeId>,
    /// Mapping from NodeId to ClientHandle
    pub node_to_handle: HashMap<NodeId, u32>,
    /// Mapping from ClientHandle to ServerID
    pub handle_to_server_id: HashMap<u32, u32>,
}

impl SubscriptionState {
    /// Register a new monitored item
    pub fn register_item(&mut self, node_id: NodeId, monitored_item_id: u32, handle: u32) {
        self.handle_to_node.insert(handle, node_id.clone());
        self.node_to_handle.insert(node_id, handle);
        self.handle_to_server_id.insert(handle, monitored_item_id);
    }

    /// Unregister a monitored item by NodeId
    pub fn unregister_by_node(&mut self, node_id: &NodeId) -> Option<u32> {
        if let Some(handle) = self.node_to_handle.remove(node_id) {
            self.handle_to_node.remove(&handle);
            self.handle_to_server_id.remove(&handle)
        } else {
            None
        }
    }

    /// Clear all state
    pub fn clear(&mut self) {
        self.subscription_id = None;
        self.handle_to_node.clear();
        self.node_to_handle.clear();
        self.handle_to_server_id.clear();
    }

    /// Get the NodeId for a ClientHandle
    pub fn get_node_id(&self, handle: u32) -> Option<&NodeId> {
        self.handle_to_node.get(&handle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monitored_data_creation() {
        let node_id = NodeId::new(2, "TestVar");
        let data = MonitoredData::new(node_id.clone(), "Test Variable".to_string());
        
        assert_eq!(data.node_id, node_id);
        assert_eq!(data.display_name, "Test Variable");
        assert!(data.value.is_none());
        assert!(!data.status.is_good());
    }

    #[test]
    fn test_variant_to_f64() {
        assert_eq!(variant_to_f64(&Variant::Int32(42)), Some(42.0));
        assert_eq!(variant_to_f64(&Variant::Float(3.14)), Some(3.14_f32 as f64));
        assert_eq!(variant_to_f64(&Variant::Boolean(true)), Some(1.0));
        assert!(variant_to_f64(&Variant::String("hello".into())).is_none());
    }

    #[test]
    fn test_subscription_state() {
        let mut state = SubscriptionState::default();
        let node_id = NodeId::new(2, "Var1");
        
        // Register with server_id=100, handle=1
        state.register_item(node_id.clone(), 100, 1);
        
        // Lookup by handle
        assert_eq!(state.get_node_id(1), Some(&node_id));
        
        // Remove by node_id, returns handle
        let removed = state.unregister_by_node(&node_id);
        assert_eq!(removed, Some(1));
        assert!(state.get_node_id(1).is_none());
    }
}
