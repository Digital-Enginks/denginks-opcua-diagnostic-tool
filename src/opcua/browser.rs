//! Node browsing functionality
//!
//! Provides Browse service operations to navigate the OPC-UA address space.

use anyhow::{Context, Result};
use std::sync::Arc;

use opcua::client::Session;
use opcua::types::{
    BrowseDescription, BrowseDirection, BrowseResultMask,
    NodeId, ReferenceTypeId,
};

/// Information about a browsed node
#[derive(Debug, Clone)]
pub struct BrowsedNode {
    /// The NodeId of this node
    pub node_id: NodeId,
    /// The browse name (namespace:name)
    pub browse_name: String,
    /// The display name (human-readable)
    pub display_name: String,
    /// The node class (Object, Variable, Method, etc.)
    pub node_class: NodeClass,
    /// Type definition node ID (if applicable)
    pub type_definition: Option<NodeId>,
    /// Whether this node has children (for lazy loading)
    pub has_children: bool,
}

/// OPC-UA Node Classes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NodeClass {
    Object,
    Variable,
    Method,
    ObjectType,
    VariableType,
    ReferenceType,
    DataType,
    View,
    Unknown,
}

impl NodeClass {
    /// Get an icon string for the node class
    pub fn icon(&self) -> &'static str {
        match self {
            NodeClass::Object => "📁",
            NodeClass::Variable => "📊",
            NodeClass::Method => "⚡",
            NodeClass::ObjectType => "📋",
            NodeClass::VariableType => "📈",
            NodeClass::ReferenceType => "🔗",
            NodeClass::DataType => "🔢",
            NodeClass::View => "👁",
            NodeClass::Unknown => "❓",
        }
    }

    /// Convert from OPC-UA node class enum
    pub fn from_opcua(node_class: opcua::types::NodeClass) -> Self {
        match node_class {
            opcua::types::NodeClass::Object => NodeClass::Object,
            opcua::types::NodeClass::Variable => NodeClass::Variable,
            opcua::types::NodeClass::Method => NodeClass::Method,
            opcua::types::NodeClass::ObjectType => NodeClass::ObjectType,
            opcua::types::NodeClass::VariableType => NodeClass::VariableType,
            opcua::types::NodeClass::ReferenceType => NodeClass::ReferenceType,
            opcua::types::NodeClass::DataType => NodeClass::DataType,
            opcua::types::NodeClass::View => NodeClass::View,
            _ => NodeClass::Unknown,
        }
    }
}

impl std::fmt::Display for NodeClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeClass::Object => write!(f, "Object"),
            NodeClass::Variable => write!(f, "Variable"),
            NodeClass::Method => write!(f, "Method"),
            NodeClass::ObjectType => write!(f, "ObjectType"),
            NodeClass::VariableType => write!(f, "VariableType"),
            NodeClass::ReferenceType => write!(f, "ReferenceType"),
            NodeClass::DataType => write!(f, "DataType"),
            NodeClass::View => write!(f, "View"),
            NodeClass::Unknown => write!(f, "Unknown"),
        }
    }
}


/// Browse a specific node and return its children
pub async fn browse_node(session: Arc<Session>, parent_node_id: &NodeId) -> Result<Vec<BrowsedNode>> {
    tracing::debug!("Browsing node: {:?}", parent_node_id);

    // Create browse description
    let browse_description = BrowseDescription {
        node_id: parent_node_id.clone(),
        browse_direction: BrowseDirection::Forward,
        reference_type_id: ReferenceTypeId::HierarchicalReferences.into(),
        include_subtypes: true,
        node_class_mask: 0xFF, // All node classes
        result_mask: BrowseResultMask::All as u32,
    };

    // Perform browse with max references and no view
    let browse_result = session
        .browse(&[browse_description], 0, None)
        .await
        .context("Browse request failed")?;

    if browse_result.is_empty() {
        return Ok(Vec::new());
    }

    let result = &browse_result[0];

    // Check status code
    if !result.status_code.is_good() {
        anyhow::bail!("Browse failed with status: {:?}", result.status_code);
    }

    // Convert references to BrowsedNode
    let nodes: Vec<BrowsedNode> = result
        .references
        .as_ref()
        .map(|refs| {
            refs.iter()
                .map(|reference| {
                    let node_class = NodeClass::from_opcua(reference.node_class);
                    
                    BrowsedNode {
                        node_id: reference.node_id.node_id.clone(),
                        browse_name: reference.browse_name.to_string(),
                        display_name: reference.display_name.text.to_string(),
                        node_class,
                        type_definition: Some(reference.type_definition.node_id.clone()),
                        has_children: matches!(node_class, NodeClass::Object | NodeClass::ObjectType | NodeClass::View),
                    }
                })
                .collect()
        })
        .unwrap_or_default();

    tracing::debug!("Found {} children for {:?}", nodes.len(), parent_node_id);

    Ok(nodes)
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_class_icons() {
        assert_eq!(NodeClass::Object.icon(), "📁");
        assert_eq!(NodeClass::Variable.icon(), "📊");
        assert_eq!(NodeClass::Method.icon(), "⚡");
    }
}
