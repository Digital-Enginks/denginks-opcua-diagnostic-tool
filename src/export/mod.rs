



use std::path::Path;
use std::fs::File;
use anyhow::{Context, Result};
use serde::Serialize;


use crate::opcua::subscription::MonitoredData;
use crate::opcua::browser::BrowsedNode;


#[derive(Serialize)]
struct ExportItem<'a> {
    name: &'a str,
    node_id: String,
    value: String,
    status: String,
    timestamp: String,
}

impl<'a> From<&'a MonitoredData> for ExportItem<'a> {
    fn from(item: &'a MonitoredData) -> Self {
        Self {
            name: &item.display_name,
            node_id: item.node_id.to_string(),
            value: item.value_string(),
            status: format!("{:?}", item.status),
            timestamp: item.timestamp_string(),
        }
    }
}


pub struct ExportEngine;

impl ExportEngine {
    
    pub fn export_watchlist_to_csv(items: &[MonitoredData], path: &Path) -> Result<()> {
        let mut wtr = csv::Writer::from_path(path)
            .context("Failed to create CSV writer")?;

        for item in items {
            let export_item = ExportItem::from(item);
            wtr.serialize(export_item)
                .context("Failed to serialize item to CSV")?;
        }

        wtr.flush().context("Failed to flush CSV writer")?;
        Ok(())
    }

    
    pub fn export_watchlist_to_json(items: &[MonitoredData], path: &Path) -> Result<()> {
        let export_items: Vec<ExportItem> = items.iter().map(ExportItem::from).collect();
        
        let file = File::create(path).context("Failed to create JSON file")?;
        serde_json::to_writer_pretty(file, &export_items)
            .context("Failed to write JSON data")?;
            
        Ok(())
    }

    
    
    pub fn export_crawl_result_to_json(nodes: &[BrowsedNode], path: &Path) -> Result<()> {
        use serde_json::{json, Map, Value};
        use crate::opcua::browser::NodeClass;
        
        
        
        let mut root = Map::new();
        
        for node in nodes {
            
            
            let name = if node.browse_name.contains(':') {
                
                node.browse_name.split(':').next_back().unwrap_or(&node.browse_name)
            } else {
                &node.browse_name
            };
            
            
            let node_entry = json!({
                "nodeId": node.node_id.to_string(),
                "displayName": node.display_name,
                "nodeClass": node.node_class.to_string()
            });
            
            
            
            match node.node_class {
                NodeClass::Object | NodeClass::ObjectType | NodeClass::View => {
                    
                    let mut obj_map = Map::new();
                    obj_map.insert("_nodeId".to_string(), Value::String(node.node_id.to_string()));
                    obj_map.insert("_nodeClass".to_string(), Value::String(node.node_class.to_string()));
                    root.insert(name.to_string(), Value::Object(obj_map));
                }
                NodeClass::Variable => {
                    
                    root.insert(name.to_string(), node_entry);
                }
                _ => {
                    
                    root.insert(name.to_string(), node_entry);
                }
            }
        }
        
        let file = File::create(path).context("Failed to create JSON file")?;
        serde_json::to_writer_pretty(file, &Value::Object(root))
            .context("Failed to write JSON data")?;

        Ok(())
    }

    
    pub fn export_crawl_result_to_csv(nodes: &[BrowsedNode], path: &Path) -> Result<()> {
        #[derive(Serialize)]
        struct CrawlNodeExport<'a> {
            node_id: String,
            browse_name: &'a str,
            display_name: &'a str,
            node_class: String,
        }

        let mut wtr = csv::Writer::from_path(path)
            .context("Failed to create CSV writer")?;

        for node in nodes {
            let export_node = CrawlNodeExport {
                node_id: node.node_id.to_string(),
                browse_name: &node.browse_name,
                display_name: &node.display_name,
                node_class: node.node_class.to_string(),
            };
            wtr.serialize(export_node)
                .context("Failed to serialize node to CSV")?;
        }

        wtr.flush().context("Failed to flush CSV writer")?;
        Ok(())
    }
}
