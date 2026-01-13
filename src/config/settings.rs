//! Application settings

use serde::{Deserialize, Serialize};

/// Application settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct Settings {
    /// Default subscription publish interval in milliseconds
    pub subscription_interval_ms: u32,
    /// Maximum number of items in watchlist
    pub max_watchlist_items: usize,
    /// Trending history duration in seconds
    pub trending_history_seconds: u32,
    /// Auto-save bookmarks on change
    pub auto_save_bookmarks: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            subscription_interval_ms: 1000,
            max_watchlist_items: 50,
            trending_history_seconds: 300,
            auto_save_bookmarks: true,
        }
    }
}
