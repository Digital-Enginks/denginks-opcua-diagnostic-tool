//! Trending chart for real-time data visualization
//!
//! Provides a line chart display for selected monitored items.

use eframe::egui;
use egui_plot::{Line, Legend, Plot, PlotPoints, AxisHints};
use opcua::types::NodeId;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use crate::opcua::subscription::MonitoredData;

/// Time window options in seconds
const TIME_WINDOWS: [u64; 4] = [30, 60, 300, 600];

/// Trending panel state
pub struct TrendingPanel {
    /// Selected time window in seconds
    time_window: u64,
}

impl Default for TrendingPanel {
    fn default() -> Self {
        Self {
            time_window: 60,
        }
    }
}

/// Generate a consistent color for a NodeId (based on hash of string representation)
/// This ensures each variable has a unique color even if display names are the same
pub fn color_for_node_id(node_id: &NodeId) -> egui::Color32 {
    let node_str = node_id.to_string();
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    node_str.hash(&mut hasher);
    let hash = hasher.finish();
    
    // Use the hash to generate HSV values with good separation
    let hue = (hash % 360) as f32 / 360.0;
    let saturation = 0.7 + (((hash >> 8) % 30) as f32 / 100.0); // 0.7-1.0
    let value = 0.8 + (((hash >> 16) % 20) as f32 / 100.0); // 0.8-1.0
    
    egui::Color32::from(egui::ecolor::Hsva::new(hue, saturation, value, 1.0))
}

/// Format a Unix timestamp as HH:MM:SS
fn format_time(timestamp: f64) -> String {
    use std::time::{UNIX_EPOCH, Duration};
    
    if let Ok(duration) = Duration::try_from_secs_f64(timestamp) {
        let time = UNIX_EPOCH + duration;
        if let Ok(elapsed) = time.duration_since(UNIX_EPOCH) {
            let secs = elapsed.as_secs();
            let hours = (secs / 3600) % 24;
            let minutes = (secs / 60) % 60;
            let seconds = secs % 60;
            return format!("{:02}:{:02}:{:02}", hours, minutes, seconds);
        }
    }
    format!("{:.0}", timestamp)
}

impl TrendingPanel {
    /// Show the trending panel
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        monitored_items: &HashMap<NodeId, MonitoredData>,
    ) {
        ui.horizontal(|ui| {
            ui.heading("📈 Live Trend");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Time window selector
                egui::ComboBox::from_id_salt("time_window")
                    .selected_text(format!("Window: {}s", self.time_window))
                    .show_ui(ui, |ui| {
                        for window in TIME_WINDOWS {
                            ui.selectable_value(&mut self.time_window, window, format!("{}s", window));
                        }
                    });
            });
        });
        
        ui.separator();

        // Prepare plot lines
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs_f64())
            .unwrap_or(0.0);

        let min_time = current_time - self.time_window as f64;
        
        // Custom X-axis formatter for human-readable time
        let x_fmt = |mark: egui_plot::GridMark, _range: &std::ops::RangeInclusive<f64>| {
            format_time(mark.value)
        };
        
        // Count how many items will be shown in trend
        let trending_items: Vec<_> = monitored_items.iter()
            .filter(|(_, item)| item.show_in_trend && item.is_trendable() && !item.history.is_empty())
            .collect();
        
        // Plot logic
        Plot::new("trend_plot")
            .legend(Legend::default())
            .x_axis_label("Time")
            .y_axis_label("Value")
            .custom_x_axes(vec![AxisHints::new_x().formatter(x_fmt)])
            .include_x(current_time)
            .include_x(min_time)
            .show(ui, |plot_ui| {
                for (node_id, item) in &trending_items {
                    // Convert history to plot points, filtering by time window
                    let points: PlotPoints = item.history
                        .iter()
                        .filter(|(t, _)| *t >= min_time)
                        .map(|(t, v)| [*t, *v])
                        .collect();

                    // Use custom color if set, otherwise generate from node_id
                    let color = if let Some(rgb) = item.trend_color {
                        egui::Color32::from_rgb(rgb[0], rgb[1], rgb[2])
                    } else {
                        color_for_node_id(node_id)
                    };

                    plot_ui.line(
                        Line::new(points)
                            .name(&item.display_name)
                            .color(color)
                            .width(2.0)
                    );
                }
            });
            
        // If no items are selected for trending, show a hint
        if trending_items.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label("Select numeric items in the Watchlist (📈) to visualize them here.\nNote: Dates and strings cannot be graphed.");
            });
        }
    }
}

