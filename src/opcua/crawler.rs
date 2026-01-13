//! Crawler for OPC-UA address space
//!
//! Recursively browses the address space to discover nodes.

use std::sync::Arc;
use std::collections::HashSet;
use std::time::Instant;
use opcua::client::Session;
use opcua::types::NodeId;
use anyhow::Result;

use crate::opcua::browser::{browse_node, BrowsedNode};

/// Configuration for the crawler
#[derive(Debug, Clone)]
pub struct CrawlConfig {
    /// Maximum depth to crawl
    pub max_depth: usize,
    /// Maximum number of nodes to collect
    pub max_nodes: usize,
    /// Start node
    pub start_node: NodeId,
}

/// Crawler implementation
pub struct Crawler {
    session: Arc<Session>,
    visited: HashSet<String>,
    results: Vec<BrowsedNode>,
    config: CrawlConfig,
}

impl Crawler {
    pub fn new(session: Arc<Session>, config: CrawlConfig) -> Self {
        Self {
            session,
            visited: HashSet::new(),
            results: Vec::new(),
            config,
        }
    }

    /// Execute the crawl
    pub async fn crawl(&mut self) -> Result<Vec<BrowsedNode>> {
        self.visited.clear();
        self.results.clear();

        tracing::info!("Starting crawl from {:?} with depth {}", self.config.start_node, self.config.max_depth);
        let start = Instant::now();

        // Start recursion
        self.crawl_recursive(&self.config.start_node.clone(), 0).await?;

        tracing::info!("Crawl finished. Found {} nodes in {:?}", self.results.len(), start.elapsed());
        Ok(self.results.clone())
    }

    #[async_recursion::async_recursion]
    async fn crawl_recursive(&mut self, node_id: &NodeId, depth: usize) -> Result<()> {
        // Check limits
        if depth >= self.config.max_depth {
            return Ok(());
        }
        // Limit Removed as per request (Phase 3)
        // if self.results.len() >= self.config.max_nodes {
        //    return Ok(());
        // }

        // Cycle detection
        let node_str = node_id.to_string();
        if self.visited.contains(&node_str) {
            return Ok(());
        }
        self.visited.insert(node_str);

        // Browse children
        match browse_node(self.session.clone(), node_id).await {
            Ok(children) => {
                for child in children {
                    // Add to results
                    self.results.push(child.clone());

                    // Recurse if it has children
                    if child.has_children {
                        self.crawl_recursive(&child.node_id, depth + 1).await?;
                    }
                    
                    if self.results.len() >= self.config.max_nodes {
                        break;
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Failed to browse node {:?}: {}", node_id, e);
            }
        }

        Ok(())
    }
}
