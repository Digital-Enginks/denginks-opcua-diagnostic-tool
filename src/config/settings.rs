use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct Settings {
    
    pub subscription_interval_ms: u32,
    
    pub max_watchlist_items: usize,
    
    pub trending_history_seconds: u32,
    
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let s = Settings::default();
        assert_eq!(s.subscription_interval_ms, 1000);
        assert_eq!(s.auto_save_bookmarks, true);
    }
}