//! Main application state and eframe App implementation

use eframe::egui;
use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::Arc;
use tokio::runtime::Handle;
use tokio::sync::RwLock;
use opcua::types::{NodeId, DataValue};

use crate::config::bookmarks::Bookmarks;
use crate::network::diagnostics::DiagnosticStep;
use crate::opcua::browser::BrowsedNode;
use crate::opcua::client::{ClientConfig, OpcUaClient};
use crate::opcua::subscription_manager::{SubscriptionManager, SubscriptionAction};
use crate::ui::connection::ConnectionPanel;
use crate::ui::error_panel::{ErrorPanel, ErrorSeverity};
use crate::ui::monitor::{MonitorPanel, MonitorAction};
use crate::ui::trending::TrendingPanel;
use crate::ui::crawler_panel::{CrawlerPanel, CrawlerAction};
use crate::ui::certificates_panel::CertificatesPanel;
use crate::ui::tree_view::TreeView;
use crate::ui::properties::PropertiesPanel;
use crate::utils::i18n::{self, T, Language};


/// Current status of the application
#[derive(Debug, Clone, PartialEq, Default)]
pub enum AppStatus {
    #[default]
    Idle,
    Busy {
        task_name: String,
        start_time: std::time::Instant,
    },
}

/// Metadata for an active background task
pub struct ActiveTask {
    /// Name of the task for UI display
    pub name: String,
    /// Handle to abort the task if possible
    pub handle: tokio::task::JoinHandle<()>,
    /// Cancellation token
    pub cancel_token: tokio_util::sync::CancellationToken,
}

/// Messages sent from background tasks to the UI
#[derive(Debug)]
pub enum BackendMessage {
    /// Session established successfully
    SessionEstablished { endpoint: String },
    /// Session closed
    SessionClosed,
    /// Browse result: (Parent NodeId, Children)
    BrowseResult(NodeId, Result<Vec<BrowsedNode>, String>),
    /// Error message to display
    Error(String),
    /// Status message update
    StatusMessage(String),
    /// Live data change: (MonitoredItemId, DataValue)
    DataChange(u32, DataValue),
    /// Subscription created
    SubscriptionCreated(u32),
    /// Monitored items added: Vec<(NodeId, MonitoredItemId, ClientHandle)>
    MonitoredItemsAdded(Vec<(NodeId, u32, u32)>),
    /// Crawl result
    CrawlResult(Result<Vec<BrowsedNode>, String>),
    /// Diagnostic step progress
    DiagnosticStep(DiagnosticStep),
    /// Diagnostic complete
    DiagnosticComplete(crate::network::diagnostics::DiagnosticResult),
}


/// Current connection state
#[derive(Debug, Clone, PartialEq, Default)]
pub enum ConnectionState {
    #[default]
    Disconnected,
    Connected { endpoint: String },
    Error(String),
}



/// Main application struct
pub struct DiagnosticApp {
    /// Tokio runtime handle for spawning async tasks
    runtime: Handle,

    /// Channel sender for sending messages to background tasks
    #[allow(dead_code)]
    task_tx: mpsc::Sender<TaskMessage>,

    /// Channel receiver for receiving messages from background tasks
    backend_rx: mpsc::Receiver<BackendMessage>,

    /// Channel sender for background tasks to send messages back
    backend_tx: mpsc::Sender<BackendMessage>,

    /// Current connection state
    connection_state: ConnectionState,

    /// Server bookmarks
    bookmarks: Bookmarks,

    /// Connection panel UI state
    connection_panel: ConnectionPanel,

    /// Show connection panel
    show_connection_panel: bool,

    /// Status bar message
    status_message: String,

    /// Active OPC-UA client session
    opcua_client: Arc<RwLock<Option<OpcUaClient>>>,

    /// Cached children nodes [Parent NodeId -> Children]
    node_cache: HashMap<NodeId, Vec<BrowsedNode>>,

    /// Top-level root nodes
    root_nodes: Vec<BrowsedNode>,

    /// Currently selected node
    selected_node: Option<BrowsedNode>,

    /// Global application status
    status: AppStatus,

    /// Currently running background task
    active_task: Option<ActiveTask>,

    /// Show about dialog
    show_about: bool,

    // -- Phase 4 Fields --
    
    /// Subscription Manager
    pub subscription_manager: SubscriptionManager,
    
    /// Watchlist (Monitor) Panel
    monitor_panel: MonitorPanel,
    
    /// Trending Panel
    trending_panel: TrendingPanel,
    
    /// Show watchlist panel
    show_watchlist: bool,
    
    /// Show trending panel
    show_trending: bool,

    // -- Phase 5 Fields --

    /// Crawler Panel
    crawler_panel: CrawlerPanel,

    /// Show crawler panel
    show_crawler: bool,

    // -- Phase 6 Fields --

    /// Certificates Panel
    certificates_panel: CertificatesPanel,

    /// Show certificates panel
    show_certificates: bool,

    /// Current language
    current_lang: Language,

    // -- Error Handling --

    /// Error notification panel
    error_panel: ErrorPanel,

    /// Show error panel
    show_errors: bool,

    /// Last connection health check timestamp
    last_connection_check: std::time::Instant,
}


/// Messages sent to background tasks
#[derive(Debug)]
#[allow(dead_code)]
pub enum TaskMessage {
    /// Check network connectivity
    CheckNetwork(String),
    /// Discover endpoints
    DiscoverEndpoints(String),
    /// Connect to server
    Connect(ClientConfig),
    /// Disconnect from server
    Disconnect,
    /// Browse a node
    Browse(NodeId),
}

impl DiagnosticApp {
    /// Create a new DiagnosticApp instance
    pub fn new(_cc: &eframe::CreationContext<'_>, runtime: Handle) -> Self {
        // Create channels for communication
        let (task_tx, _task_rx) = std::sync::mpsc::channel::<TaskMessage>();
        let (backend_tx, backend_rx) = std::sync::mpsc::channel::<BackendMessage>();

        // Load bookmarks
        let bookmarks = Bookmarks::load().unwrap_or_default();

        Self {
            runtime,
            task_tx,
            backend_rx,
            backend_tx,
            connection_state: ConnectionState::default(),
            bookmarks,
            connection_panel: ConnectionPanel::default(),
            show_connection_panel: true,
            status_message: i18n::t(T::ReadyNotConnected, Language::default()).to_string(),
            opcua_client: Arc::new(RwLock::new(None)),
            node_cache: HashMap::new(),
            root_nodes: Vec::new(),
            selected_node: None,
            status: AppStatus::Idle,
            active_task: None,
            show_about: false,
            // Phase 4
            // Phase 4
            subscription_manager: SubscriptionManager::new(),
            monitor_panel: MonitorPanel,
            trending_panel: TrendingPanel::default(),
            show_watchlist: true,
            show_trending: true,
            // Phase 5
            crawler_panel: CrawlerPanel::default(),
            show_crawler: false,
            // Phase 6
            certificates_panel: CertificatesPanel::default(),
            show_certificates: false,
            // i18n
            current_lang: Language::default(),
            // Error handling
            error_panel: ErrorPanel::default(),
            show_errors: false,
            last_connection_check: std::time::Instant::now(),
        }

    }

    /// Process messages from background tasks
    fn process_backend_messages(&mut self) {
        while let Ok(msg) = self.backend_rx.try_recv() {
            match msg {
                BackendMessage::SessionEstablished { endpoint } => {
                    self.connection_state = ConnectionState::Connected { endpoint: endpoint.clone() };
                    self.status_message = i18n::t(T::ConnectedTo, self.current_lang).replace("{}", &endpoint);
                    self.connection_panel.set_connecting(false);
                    
                    // Auto-hide connection panel on successful connection
                    self.show_connection_panel = false;
                    
                    // Reset state
                    self.root_nodes.clear();
                    self.node_cache.clear();
                    self.selected_node = None;
                    self.subscription_manager.clear();

                    // Auto-browse root on connect
                    self.browse_node(NodeId::from(opcua::types::ObjectId::RootFolder));
                }
                BackendMessage::SessionClosed => {
                    self.connection_state = ConnectionState::Disconnected;
                    self.status_message = i18n::t(T::Disconnected, self.current_lang).to_string();
                    self.connection_panel.set_connecting(false);
                    self.root_nodes.clear();
                    self.node_cache.clear();
                    self.selected_node = None;
                    self.subscription_manager.clear();
                    
                    // Show connection panel again so user can reconnect
                    self.show_connection_panel = true;
                    
                    // Notify user about disconnection
                    self.error_panel.add_error(
                        i18n::t(T::ServerDisconnected, self.current_lang),
                        ErrorSeverity::Warning
                    );
                }
                BackendMessage::BrowseResult(parent_id, result) => {
                    match result {
                        Ok(nodes) => {
                            if parent_id == opcua::types::ObjectId::RootFolder {
                                self.root_nodes = nodes;
                            } else {
                                self.node_cache.insert(parent_id, nodes);
                            }
                        }
                        Err(e) => {
                            self.status_message = format!("Browse error: {}", e);
                        }
                    }
                }
                BackendMessage::Error(e) => {
                    self.connection_state = ConnectionState::Error(e.clone());
                    self.status_message = format!("Error: {}", e);
                    self.connection_panel.set_connecting(false);
                    self.subscription_manager.creating_subscription = false;
                    
                    // Add error notification
                    self.error_panel.add_error(&e, ErrorSeverity::Error);
                }
                BackendMessage::StatusMessage(msg) => {
                    self.status_message = msg;
                }
                BackendMessage::DataChange(item_id, value) => {
                    self.subscription_manager.handle_data_change(item_id, value);
                }
                BackendMessage::SubscriptionCreated(id) => {
                    self.subscription_manager.subscription_state.subscription_id = Some(id);
                    self.subscription_manager.creating_subscription = false;
                    
                    // Add any pending items
                    self.subscription_manager.spawn_add_items_task(
                        &self.runtime,
                        self.opcua_client.clone(),
                        self.backend_tx.clone()
                    );
                }
                BackendMessage::MonitoredItemsAdded(pairs) => {
                    self.subscription_manager.handle_monitored_items_added(pairs);
                }
                BackendMessage::CrawlResult(result) => {
                    self.crawler_panel.is_crawling = false;
                    match result {
                        Ok(nodes) => {
                            self.crawler_panel.results = nodes;
                            self.crawler_panel.status = i18n::t(T::CrawlComplete, self.current_lang).replace("{}", &self.crawler_panel.results.len().to_string());
                        }
                        Err(e) => {
                            self.crawler_panel.status = i18n::t(T::CrawlFailed, self.current_lang).replace("{}", &e);
                        }
                    }
                }
                BackendMessage::DiagnosticStep(step) => {
                    self.connection_panel.add_diagnostic_step(step);
                }
                BackendMessage::DiagnosticComplete(result) => {
                    self.connection_panel.set_diagnostic_result(result);
                    // Clear the active task since diagnostic is done
                    if let Some(task) = &self.active_task {
                        if task.name == i18n::t(T::Diagnose, self.current_lang) {
                            self.active_task = None;
                            self.status = AppStatus::Idle;
                        }
                    }
                }
            }
        }

        // Check if active task has finished naturally or panicked
        if let Some(task) = &self.active_task {
            if task.handle.is_finished() {
                // If it finished but we didn't get a specific success/fail message affecting state,
                // we must ensure the UI is not stuck in a "loading" state.
                self.connection_panel.set_connecting(false);
                
                self.active_task = None;
                self.status = AppStatus::Idle;
            }
        }

        // Connection health check - every 2 seconds
        if self.last_connection_check.elapsed().as_secs() >= 2 {
            self.last_connection_check = std::time::Instant::now();
            self.check_connection_health();
        }
    }

    /// Check if the connection is still alive
    fn check_connection_health(&mut self) {
        if let ConnectionState::Connected { .. } = &self.connection_state {
            let client_handle = self.opcua_client.clone();
            let tx = self.backend_tx.clone();
            
            self.runtime.spawn(async move {
                let guard = client_handle.read().await;
                if let Some(client) = guard.as_ref() {
                    if !client.is_connected() {
                        // Connection lost
                        let _ = tx.send(BackendMessage::SessionClosed);
                    }
                } else {
                    // No client = disconnected
                    let _ = tx.send(BackendMessage::SessionClosed);
                }
            });
        }
    }

    /// Set the application to busy state with a task name
    pub fn set_busy(&mut self, task_name: &str, handle: tokio::task::JoinHandle<()>, cancel_token: tokio_util::sync::CancellationToken) {
        self.status = AppStatus::Busy {
            task_name: task_name.to_string(),
            start_time: std::time::Instant::now(),
        };
        self.active_task = Some(ActiveTask {
            name: task_name.to_string(),
            handle,
            cancel_token,
        });
    }

    /// Set the application to busy state with a task name (simple version for backward compat)
    pub fn set_busy_simple(&mut self, task_name: &str, handle: tokio::task::JoinHandle<()>) {
        let cancel_token = tokio_util::sync::CancellationToken::new();
        self.set_busy(task_name, handle, cancel_token);
    }

    /// Cancel the active task
    pub fn cancel_task(&mut self) {
        if let Some(task) = self.active_task.take() {
            // Signal cancellation first
            task.cancel_token.cancel();
            // Then abort the handle
            task.handle.abort();
            self.status = AppStatus::Idle;
            self.status_message = i18n::t(T::TaskCancelled, self.current_lang).replace("{}", &task.name);
            // Also reset connection panel state
            self.connection_panel.reset_diagnostic();
            self.connection_panel.set_connecting(false);
        }
    }

    /// Get the tokio runtime handle
    #[allow(dead_code)]
    pub fn runtime(&self) -> &Handle {
        &self.runtime
    }

    /// Get the backend message sender for async tasks
    #[allow(dead_code)]
    pub fn backend_sender(&self) -> mpsc::Sender<BackendMessage> {
        self.backend_tx.clone()
    }

    /// Get the OPC-UA client handle
    #[allow(dead_code)]
    pub fn opcua_client(&self) -> Arc<RwLock<Option<OpcUaClient>>> {
        self.opcua_client.clone()
    }

    /// Check if currently connected
    pub fn is_connected(&self) -> bool {
        matches!(self.connection_state, ConnectionState::Connected { .. })
    }

    /// Connect to an OPC-UA server
    pub fn connect(&mut self, config: ClientConfig) {
        if let Err(e) = crate::network::precheck::parse_endpoint_url(&config.endpoint_url) {
            self.status_message = format!("{}: {}", i18n::t(T::ConnectionError, self.current_lang), e);
            self.connection_state = ConnectionState::Error(e);
            return;
        }
        self.status_message = i18n::t(T::Connecting, self.current_lang).to_string();
        self.connection_panel.set_connecting(true);
        
        let tx = self.backend_tx.clone();
        let client_handle = self.opcua_client.clone();
        let endpoint = config.endpoint_url.clone();

        let handle = self.runtime.spawn(async move {
            let _ = tx.send(BackendMessage::StatusMessage(i18n::t(T::EstablishingConnection, Language::default()).to_string()));

            match OpcUaClient::connect(config).await {
                Ok(client) => {
                    // Store the client
                    {
                        let mut guard = client_handle.write().await;
                        *guard = Some(client);
                    }
                    let _ = tx.send(BackendMessage::SessionEstablished { endpoint });
                }
                Err(e) => {
                    let _ = tx.send(BackendMessage::Error(format!("Connection failed: {}", e)));
                }
            }
        });

        self.set_busy_simple(i18n::t(T::Connecting, self.current_lang), handle);
    }

    /// Disconnect from the current server
    pub fn disconnect(&mut self) {
        let tx = self.backend_tx.clone();
        let client_handle = self.opcua_client.clone();

        self.runtime.spawn(async move {
            let mut guard = client_handle.write().await;
            if let Some(client) = guard.take() {
                client.disconnect().await;
            }
            let _ = tx.send(BackendMessage::SessionClosed);
        });
    }

    /// Browse a specific node
    fn browse_node(&mut self, node_id: NodeId) {
        let tx = self.backend_tx.clone();
        let client_handle = self.opcua_client.clone();
        let request_id = node_id.clone();

        let handle = self.runtime.spawn(async move {
            let guard = client_handle.read().await;
            if let Some(client) = guard.as_ref() {
                let session = client.session();
                match crate::opcua::browser::browse_node(session, &node_id).await {
                    Ok(nodes) => {
                        let _ = tx.send(BackendMessage::BrowseResult(request_id, Ok(nodes)));
                    }
                    Err(e) => {
                        let _ = tx.send(BackendMessage::BrowseResult(request_id, Err(e.to_string())));
                    }
                }
            }
        });

        self.set_busy_simple(i18n::t(T::Properties, self.current_lang), handle);
    }

    /// Start network diagnostic for a server input
    pub fn start_diagnostic(&mut self, input: String) {
        self.connection_panel.start_diagnostic();
        
        let tx = self.backend_tx.clone();
        let cancel_token = tokio_util::sync::CancellationToken::new();
        let cancel_token_clone = cancel_token.clone();
        let lang = self.current_lang;
        
        let (progress_tx, mut progress_rx) = tokio::sync::mpsc::channel::<crate::network::diagnostics::DiagnosticStep>(32);
        
        // Spawn a forwarder task to send progress updates to the UI
        let tx_progress = tx.clone();
        self.runtime.spawn(async move {
            while let Some(step) = progress_rx.recv().await {
                let _ = tx_progress.send(BackendMessage::DiagnosticStep(step));
            }
        });
        
        let handle = self.runtime.spawn(async move {
            let result = crate::network::diagnostics::run_diagnostic(
                &input,
                progress_tx,
                cancel_token_clone,
                lang,
            ).await;
            
            let _ = tx.send(BackendMessage::DiagnosticComplete(result));
        });
        
        self.set_busy(i18n::t(T::Diagnose, self.current_lang), handle, cancel_token);
    }

    /// Add a node to the watchlist
    pub fn add_to_watchlist(&mut self, node: &BrowsedNode) {
        match self.subscription_manager.request_add_to_watchlist(node) {
            SubscriptionAction::None => {}
            SubscriptionAction::CreateSubscription => {
                self.subscription_manager.spawn_subscription_task(
                    &self.runtime,
                    self.opcua_client.clone(),
                    self.backend_tx.clone()
                );
            }
            SubscriptionAction::AddItems(items) => {
                self.subscription_manager.spawn_add_specific_items_task(
                    items,
                    &self.runtime,
                    self.opcua_client.clone(),
                    self.backend_tx.clone()
                );
            }
        }
    }

    /// Remove a node from the watchlist
    pub fn remove_from_watchlist(&mut self, node_id: &NodeId) {
        self.subscription_manager.remove_from_watchlist(
            node_id,
            &self.runtime,
            self.opcua_client.clone()
        );
    }
    
    /// Toggle trending for a node
    pub fn toggle_trending(&mut self, node_id: NodeId) {
        if let Some(item) = self.subscription_manager.monitored_items.get_mut(&node_id) {
            item.show_in_trend = !item.show_in_trend;
            if item.show_in_trend {
                 self.show_trending = true;
            }
        }
    }

    /// Change the trend color for a node
    pub fn change_trend_color(&mut self, node_id: NodeId, rgb: [u8; 3]) {
        if let Some(item) = self.subscription_manager.monitored_items.get_mut(&node_id) {
            item.trend_color = Some(rgb);
        }
    }


    /// Start a crawl task
    pub fn start_crawl(&mut self, config: crate::opcua::crawler::CrawlConfig) {
         let tx = self.backend_tx.clone();
         let client_handle = self.opcua_client.clone();

         let handle = self.runtime.spawn(async move {
             let guard = client_handle.read().await;
             if let Some(client) = guard.as_ref() {
                 let session = client.session();
                 let mut crawler = crate::opcua::crawler::Crawler::new(session, config);
                 match crawler.crawl().await {
                     Ok(nodes) => {
                         let _ = tx.send(BackendMessage::CrawlResult(Ok(nodes)));
                     },
                     Err(e) => {
                         let _ = tx.send(BackendMessage::CrawlResult(Err(e.to_string())));
                     }
                 }
             }
         });
         
         self.set_busy_simple("Crawling", handle);
    }

      /// Export watchlist to CSV
      pub fn export_watchlist_csv(&self) {
           if let Some(path) = rfd::FileDialog::new()
                .set_file_name("watchlist.csv")
                .add_filter("CSV", &["csv"])
                .save_file() 
            {
               let items: Vec<_> = self.subscription_manager.monitored_items.values().cloned().collect();
               if let Err(e) = crate::export::ExportEngine::export_watchlist_to_csv(&items, &path) {
                  eprintln!("Export failed: {}", e);
               }
           }
      }

      /// Export watchlist to JSON
      pub fn export_watchlist_json(&self) {
           if let Some(path) = rfd::FileDialog::new()
                .set_file_name("watchlist.json")
                .add_filter("JSON", &["json"])
                .save_file() 
            {
               let items: Vec<_> = self.subscription_manager.monitored_items.values().cloned().collect();
               if let Err(e) = crate::export::ExportEngine::export_watchlist_to_json(&items, &path) {
                  eprintln!("Export failed: {}", e);
               }
           }
      }

     /// Export crawl results to JSON
     pub fn export_crawl_json(&self) {
          if let Some(path) = rfd::FileDialog::new()
                .set_file_name("crawl_result.json")
                .add_filter("JSON", &["json"])
                .save_file() 
          {
              if let Err(e) = crate::export::ExportEngine::export_crawl_result_to_json(&self.crawler_panel.results, &path) {
                 eprintln!("Export failed: {}", e);
              }
          }
     }

     /// Export crawl results to CSV
     pub fn export_crawl_csv(&self) {
          if let Some(path) = rfd::FileDialog::new()
                .set_file_name("crawl_result.csv")
                .add_filter("CSV", &["csv"])
                .save_file() 
          {
              if let Err(e) = crate::export::ExportEngine::export_crawl_result_to_csv(&self.crawler_panel.results, &path) {
                 eprintln!("Export failed: {}", e);
              }
          }
     }

}

impl eframe::App for DiagnosticApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Process any pending messages from background tasks
        self.process_backend_messages();

        // Request repaint to check for new messages
        ctx.request_repaint_after(std::time::Duration::from_millis(100));

        // Apply dark theme
        ctx.set_visuals(egui::Visuals::dark());

        // Calculate elapsed time and busy state for UI
        let (elapsed_str, can_cancel) = if let AppStatus::Busy { start_time, .. } = &self.status {
            let elapsed = start_time.elapsed().as_secs();
            (Some(format!("({}s)", elapsed)), true)
        } else {
            (None, false)
        };

        // Top menu bar
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button(i18n::t(T::File, self.current_lang), |ui| {
                    if ui.button(i18n::t(T::Exit, self.current_lang)).clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                ui.menu_button(i18n::t(T::View, self.current_lang), |ui| {
                    ui.checkbox(&mut self.show_connection_panel, i18n::t(T::Connection, self.current_lang));
                    ui.checkbox(&mut self.show_watchlist, i18n::t(T::Watchlist, self.current_lang));
                    ui.checkbox(&mut self.show_trending, i18n::t(T::Trend, self.current_lang));
                    ui.checkbox(&mut self.show_crawler, i18n::t(T::Crawler, self.current_lang));
                    ui.checkbox(&mut self.show_certificates, i18n::t(T::Certificates, self.current_lang));
                    ui.checkbox(&mut self.show_errors, i18n::t(T::ErrorPanel, self.current_lang));
                    
                    ui.separator();
                    ui.label("Language / Idioma");
                    if ui.selectable_label(self.current_lang == Language::English, "English").clicked() {
                        self.current_lang = Language::English;
                    }
                    if ui.selectable_label(self.current_lang == Language::Spanish, "Español").clicked() {
                        self.current_lang = Language::Spanish;
                    }
                });

                ui.menu_button(i18n::t(T::Help, self.current_lang), |ui| {
                    if ui.button(i18n::t(T::About, self.current_lang)).clicked() {
                        self.show_about = true;
                    }
                });
            });
        });

        // About Dialog
        if self.show_about {
            egui::Window::new(i18n::t(T::AboutTitle, self.current_lang))
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading("DENGINKS OPC-UA Diagnostic Tool");
                        ui.label(egui::RichText::new(i18n::t(T::AboutVersion, self.current_lang)).strong());
                        ui.add_space(10.0);
                        ui.separator();
                        ui.add_space(10.0);
                        ui.label(i18n::t(T::AboutAuthor, self.current_lang));
                        ui.label(i18n::t(T::AboutCompany, self.current_lang));
                        ui.label(i18n::t(T::AboutYear, self.current_lang));
                        ui.add_space(20.0);
                        if ui.button(i18n::t(T::Close, self.current_lang)).clicked() {
                            self.show_about = false;
                        }
                    });
                });
        }

        // Status bar at bottom
        egui::TopBottomPanel::bottom("status_bar")
            .min_height(24.0)
            .show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Connection status indicator
                let (color, text) = match &self.connection_state {
                    ConnectionState::Disconnected => {
                        if matches!(self.status, AppStatus::Busy { ref task_name, .. } if task_name == i18n::t(T::Connecting, self.current_lang)) {
                            (egui::Color32::from_rgb(255, 255, 0), "🟡")
                        } else {
                            (egui::Color32::from_rgb(100, 100, 100), "⚫")
                        }
                    }
                    ConnectionState::Connected { .. } => (egui::Color32::from_rgb(0, 255, 0), "🟢"),
                    ConnectionState::Error(_) => (egui::Color32::from_rgb(255, 0, 0), "🔴"),
                };
                
                ui.label(egui::RichText::new(text).color(color));
                ui.separator();
                
                // Show busy status if applicable
                if let AppStatus::Busy { task_name, start_time } = &self.status {
                    let elapsed = start_time.elapsed().as_secs();
                    ui.spinner();
                    ui.label(format!("{}: {}s", task_name, elapsed));
                    ui.separator();
                    if ui.button("✕").on_hover_text("Cancel Task").clicked() {
                        self.cancel_task();
                    }
                    ui.separator();
                }

                ui.label(&self.status_message);
            });
        });

        // Connection panel (left side)
        if self.show_connection_panel {
            egui::SidePanel::left("connection_panel")
                .resizable(true)
                .default_width(320.0)
                .min_width(280.0)
                .max_width(400.0)
                .show(ctx, |ui| {
                    // Clone values needed for the closure
                    let runtime = self.runtime.clone();
                    let tx = self.backend_tx.clone();
                    let is_connected = self.is_connected();
                    let app_busy = matches!(self.status, AppStatus::Busy { .. });
                    
                    // Collect connect/disconnect actions
                    let (action, _unused_disconnect) = self.connection_panel.show(
                        ui,
                        &mut self.bookmarks,
                        elapsed_str,
                        can_cancel,
                        &runtime,
                        tx,
                        is_connected,
                        app_busy,
                        self.current_lang,
                    );

                    // Handle actions outside the panel show
                    match action {
                        Some(crate::ui::connection::ConnectionAction::Connect(config)) => {
                            self.connect(config);
                        }
                        Some(crate::ui::connection::ConnectionAction::Disconnect) => {
                            self.disconnect();
                        }
                        Some(crate::ui::connection::ConnectionAction::StartDiagnostic(input)) => {
                            self.start_diagnostic(input);
                        }
                        Some(crate::ui::connection::ConnectionAction::CancelDiagnostic) => {
                            self.cancel_task();
                        }
                        None => {}
                    }
                });
        }

        // Properties Panel (Right side)
        let mut properties_action = None;
        if self.is_connected() {
            egui::SidePanel::right("properties_panel")
                .resizable(true)
                .default_width(300.0)
                .min_width(200.0)
                .max_width(500.0)
                .show(ctx, |ui| {
                    let monitored_data = self.selected_node.as_ref()
                        .and_then(|node| self.subscription_manager.monitored_items.get(&node.node_id));
                    
                    let panel = PropertiesPanel::new(&self.selected_node, monitored_data);
                    properties_action = panel.show(ui, self.current_lang);
                });
        }
        
        // Crawler Panel (Right side, stacked or conditional)
        let mut crawler_action = None;
        if self.show_crawler {
             egui::SidePanel::right("crawler_panel")
                .resizable(true)
                .default_width(320.0)
                .min_width(250.0)
                .max_width(500.0)
                .show(ctx, |ui| {
                    crawler_action = self.crawler_panel.show(ui, self.is_connected(), self.current_lang);
                });
        }

        // Certificates Panel
        if self.show_certificates {
            egui::SidePanel::right("certificates_panel_view")
                .resizable(true)
                .default_width(320.0)
                .min_width(250.0)
                .max_width(500.0)
                .show(ctx, |ui| {
                    if let Some(action) = self.certificates_panel.show(ui, self.current_lang) {
                        self.certificates_panel.handle_action(&action);
                    }
                });
        }

        
        // Handle Crawler Actions
        if let Some(action) = crawler_action {
            match action {
                CrawlerAction::StartCrawl(config) => self.start_crawl(config),
                CrawlerAction::ExportJson => self.export_crawl_json(),
                CrawlerAction::ExportCsv => self.export_crawl_csv(),
                CrawlerAction::JumpToNode(node_id) => {
                    // Browse to the node (expand path)
                    // Simplified: just set selected node if we have it?
                    // We need to browse to it. For now, just browse it.
                    self.browse_node(node_id);
                }
            }
        }


        // Handle Properties Actions

        if let Some(action) = properties_action {
            match action {
                crate::ui::properties::PropertiesAction::AddToWatchlist(node) => {
                    self.add_to_watchlist(&node);
                }
            }
        }

        // Monitor & Trending Panel (Bottom)
        // Only show if connected AND there are items to display
        if self.is_connected() && (self.show_watchlist || self.show_trending)
           && !self.subscription_manager.monitored_items.is_empty() {
            egui::TopBottomPanel::bottom("monitor_panel")
                .resizable(true)
                .min_height(200.0)
                .max_height(500.0)
                .default_height(300.0)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut self.show_watchlist, true, format!("📊 {}", i18n::t(T::Watchlist, self.current_lang)));
                        ui.selectable_value(&mut self.show_trending, true, format!("📈 {}", i18n::t(T::Trending, self.current_lang)));
                    });
                    ui.separator();

                    // Vertical layout: Watchlist on top, Trending below
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        if self.show_watchlist {
                            if let Some(action) = self.monitor_panel.show(ui, &self.subscription_manager.monitored_items, self.current_lang) {
                                match action {
                                    MonitorAction::Remove(node_id) => self.remove_from_watchlist(&node_id),
                                    MonitorAction::ToggleTrend(node_id) => self.toggle_trending(node_id),
                                    MonitorAction::ChangeColor(node_id, rgb) => self.change_trend_color(node_id, rgb),
                                    MonitorAction::ExportCsv => self.export_watchlist_csv(),
                                    MonitorAction::ExportJson => self.export_watchlist_json(),
                                }
                            }
                            if self.show_trending {
                                ui.add_space(10.0);
                                ui.separator();
                            }
                        }
                        
                        if self.show_trending {
                            self.trending_panel.show(ui, &self.subscription_manager.monitored_items);
                        }
                    });
                });
        }

        // Error Panel (Right side)
        if self.show_errors {
            egui::SidePanel::right("error_panel")
                .resizable(true)
                .default_width(350.0)
                .min_width(280.0)
                .max_width(500.0)
                .show(ctx, |ui| {
                    self.error_panel.show_panel(ui, self.current_lang);
                });
        }

        // Show toast notifications (overlay)
        self.error_panel.show_toasts(ctx);


        // Main central panel (Tree View)
        egui::CentralPanel::default().show(ctx, |ui| {
            
            match &self.connection_state {
                ConnectionState::Connected { endpoint } => {
                    ui.label(format!("Connected to: {}", endpoint));
                    ui.separator();
                    
                    // Tree View
                    egui::ScrollArea::both()
                        .auto_shrink([false; 2])
                        .show(ui, |ui| {
                         let selected_id = self.selected_node.as_ref().map(|n| n.node_id.clone());
                         let tree = TreeView::new(&self.node_cache, &selected_id);
                         let actions = tree.show(ui, &self.root_nodes, self.current_lang);

                         for action in actions {
                             match action {
                                 crate::ui::tree_view::TreeViewAction::Select(node) => {
                                     self.selected_node = Some(node);
                                 }
                                 crate::ui::tree_view::TreeViewAction::Expand(node_id) => {
                                     self.browse_node(node_id);
                                 }
                                 crate::ui::tree_view::TreeViewAction::AddToWatchlist(node) => {
                                     self.add_to_watchlist(&node);
                                 }
                                 crate::ui::tree_view::TreeViewAction::ExportJson(node) => {
                                     // Configure crawler to start from this node
                                     self.show_crawler = true;
                                     self.crawler_panel.config.start_node = node.node_id.clone();
                                     self.crawler_panel.config.max_depth = 10; // set strictly high
                                     self.crawler_panel.config.max_nodes = 100000;
                                     
                                     // Trigger crawl
                                     self.start_crawl(self.crawler_panel.config.clone());
                                 }
                                 crate::ui::tree_view::TreeViewAction::ExportCsv(node) => {
                                      // Same as JSON, just start crawl. User will export from panel.
                                      // Ideally we'd auto-export but that requires callback hell or state machine.
                                     self.show_crawler = true;
                                     self.crawler_panel.config.start_node = node.node_id.clone();
                                     self.crawler_panel.config.max_depth = 10;
                                     self.crawler_panel.config.max_nodes = 100000;
                                     self.start_crawl(self.crawler_panel.config.clone());
                                 }
                             }
                         }
                    });
                }
                _ if matches!(self.status, AppStatus::Busy { ref task_name, .. } if task_name == i18n::t(T::Connecting, self.current_lang)) => {
                    ui.centered_and_justified(|ui| {
                        ui.vertical_centered(|ui| {
                            ui.spinner();
                            ui.add_space(10.0);
                            ui.label(i18n::t(T::ConnectingToServer, self.current_lang));
                        });
                    });
                }
                ConnectionState::Error(e) => {
                    ui.centered_and_justified(|ui| {
                        ui.vertical_centered(|ui| {
                            ui.colored_label(egui::Color32::RED, format!("⚠️ {}", i18n::t(T::ConnectionError, self.current_lang)));
                            ui.add_space(10.0);
                            ui.label(e);
                        });
                    });
                }
                ConnectionState::Disconnected => {
                    ui.centered_and_justified(|ui| {
                        ui.vertical_centered(|ui| {
                            ui.heading(egui::RichText::new(i18n::t(T::Welcome, self.current_lang)).size(24.0));
                            ui.add_space(20.0);
                            ui.label(egui::RichText::new(format!("👈 {}", i18n::t(T::StartInstructions, self.current_lang))).size(16.0));
                            ui.add_space(10.0);
                            ui.label(i18n::t(T::ConnectStep1, self.current_lang));
                            ui.label(i18n::t(T::ConnectStep2, self.current_lang));
                            ui.label(i18n::t(T::ConnectStep3, self.current_lang));
                            ui.add_space(30.0);
                            ui.label(egui::RichText::new(format!("⚠️ {}", i18n::t(T::SafetyMode, self.current_lang))).color(egui::Color32::YELLOW));
                            ui.label(i18n::t(T::ProductionSafe, self.current_lang));
                        });
                    });
                }
            }
        });
    }
}
