//! OPC-UA Client builder and session management
//!
//! Provides async client creation, session establishment with keep-alive,
//! and channel-based communication with the UI.

use anyhow::{Context, Result};
use std::sync::Arc;
use tokio::task::JoinHandle;
use std::sync::atomic::{AtomicU32, Ordering};

use opcua::client::{Client, ClientBuilder, IdentityToken, Session, Password, MonitoredItem};
use opcua::types::{EndpointDescription, MessageSecurityMode as OpcMessageSecurityMode, UserTokenPolicy, UserTokenType, StatusCode, NodeId, DataValue};

use crate::config::bookmarks::{AuthMethod, MessageSecurityMode, SecurityPolicy, ServerBookmark};
use crate::opcua::certificates::CertificateManager;

static NEXT_CLIENT_HANDLE: AtomicU32 = AtomicU32::new(1);

/// Configuration for OPC-UA client connection
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// Endpoint URL (opc.tcp://...)
    pub endpoint_url: String,
    /// Security policy
    pub security_policy: SecurityPolicy,
    /// Message security mode
    pub security_mode: MessageSecurityMode,
    /// Authentication method
    pub auth_method: AuthMethod,
}

impl ClientConfig {
    /// Create from a server bookmark
    #[allow(dead_code)]
    pub fn from_bookmark(bookmark: &ServerBookmark) -> Self {
        Self {
            endpoint_url: bookmark.endpoint_url.clone(),
            security_policy: bookmark.security_policy.clone(),
            security_mode: bookmark.security_mode.clone(),
            auth_method: bookmark.auth_method.clone(),
        }
    }

    /// Get the identity token for authentication
    pub fn identity_token(&self) -> IdentityToken {
        match &self.auth_method {
            AuthMethod::Anonymous => IdentityToken::Anonymous,
            AuthMethod::UserPassword { username, password } => {
                IdentityToken::UserName(username.clone(), Password::from(password.clone()))
            }
        }
    }

    /// Get the security policy string for endpoint matching
    pub fn security_policy_string(&self) -> &'static str {
        match self.security_policy {
            SecurityPolicy::None => "None",
            SecurityPolicy::Basic128Rsa15 => "Basic128Rsa15",
            SecurityPolicy::Basic256 => "Basic256",
            SecurityPolicy::Basic256Sha256 => "Basic256Sha256",
            SecurityPolicy::Aes128Sha256RsaOaep => "Aes128_Sha256_RsaOaep",
            SecurityPolicy::Aes256Sha256RsaPss => "Aes256_Sha256_RsaPss",
        }
    }

    /// Convert message security mode to opcua crate enum
    pub fn opcua_message_security_mode(&self) -> OpcMessageSecurityMode {
        match self.security_mode {
            MessageSecurityMode::None => OpcMessageSecurityMode::None,
            MessageSecurityMode::Sign => OpcMessageSecurityMode::Sign,
            MessageSecurityMode::SignAndEncrypt => OpcMessageSecurityMode::SignAndEncrypt,
        }
    }

    /// Get user token policy based on auth method
    pub fn user_token_policy(&self) -> UserTokenPolicy {
        match &self.auth_method {
            AuthMethod::Anonymous => UserTokenPolicy::anonymous(),
            AuthMethod::UserPassword { .. } => UserTokenPolicy {
                policy_id: "username_password".into(),
                token_type: UserTokenType::UserName,
                issued_token_type: Default::default(),
                issuer_endpoint_url: Default::default(),
                security_policy_uri: Default::default(),
            },
        }
    }
}

/// OPC-UA client wrapper with session management
pub struct OpcUaClient {
    /// The underlying OPC-UA client
    #[allow(dead_code)]
    client: Client,
    /// The active session (if connected)
    session: Arc<Session>,
    /// Event loop handle
    #[allow(dead_code)]
    event_loop_handle: JoinHandle<StatusCode>,
}

impl OpcUaClient {
    /// Create and connect a new OPC-UA client
    pub async fn connect(config: ClientConfig) -> Result<Self> {
        tracing::info!("Connecting to OPC-UA server: {}", config.endpoint_url);

        // Ensure PKI directory structure exists
        let cert_manager = CertificateManager::new()?;
        cert_manager.ensure_pki_structure()?;

        // Build the client with auto-generated keypair
        let mut client = ClientBuilder::new()
            .application_name("DengInks OPC-UA Diagnostic Tool")
            .application_uri("urn:DengInks:OpcUaDiagnostic")
            .product_uri("urn:DengInks:OpcUaDiagnostic")
            .pki_dir(cert_manager.pki_directory())
            .create_sample_keypair(true)  // Auto-generate client certificate
            .trust_server_certs(true)     // Trust all server certs for now (simplified)
            .session_retry_limit(3)
            .session_timeout(30000)
            .client()
            .map_err(|e| anyhow::anyhow!("Failed to build client: {:?}", e))?;

        // Create endpoint description from configuration
        let endpoint: EndpointDescription = (
            config.endpoint_url.as_str(),
            config.security_policy_string(),
            config.opcua_message_security_mode(),
            config.user_token_policy(),
        ).into();

        tracing::info!("Connecting to endpoint: {:?}", endpoint.endpoint_url);

        // Connect to matching endpoint
        let (session, event_loop) = client
            .connect_to_matching_endpoint(endpoint, config.identity_token())
            .await
            .context("Failed to connect to endpoint")?;

        // Spawn the event loop
        let event_loop_handle = event_loop.spawn();

        // Wait for connection to be established
        session.wait_for_connection().await;

        tracing::info!("OPC-UA session established successfully");

        Ok(Self {
            client,
            session,
            event_loop_handle,
        })
    }

    /// Disconnect from the server
    pub async fn disconnect(&self) {
        tracing::info!("Disconnecting from OPC-UA server...");
        let _ = self.session.disconnect().await;
        tracing::info!("Disconnected successfully");
    }

    /// Get a reference to the session for operations
    pub fn session(&self) -> Arc<Session> {
        self.session.clone()
    }

    /// Check if the session is still connected
    /// Note: This checks if the session object exists; actual connection state
    /// may need to be verified through a session service call
    pub fn is_connected(&self) -> bool {
        // The session object exists, assume connected unless we get an error
        // The connection_state is checked via keepalives in the event loop
        true
    }

    /// Create a subscription for monitoring items
    /// Returns the subscription ID
    pub async fn create_subscription<F>(
        &self,
        publishing_interval: std::time::Duration,
        callback: F,
    ) -> Result<u32>
    where
        F: Fn(DataValue, &MonitoredItem) + Send + Sync + 'static,
    {
        use opcua::client::DataChangeCallback;

        tracing::info!("Creating subscription with interval {:?}", publishing_interval);

        let subscription_id = self.session
            .create_subscription(
                publishing_interval,
                10,     // Lifetime count
                30,     // Max keepalive count
                0,      // Max notifications per publish (0 = unlimited)
                0,      // Priority
                true,   // Publishing enabled
                DataChangeCallback::new(callback),
            )
            .await
            .context("Failed to create subscription")?;

        tracing::info!("Created subscription with ID: {}", subscription_id);
        Ok(subscription_id)
    }



    /// Add monitored items to an existing subscription
    /// Returns a vector of (NodeId, MonitoredItemId, ClientHandle) pairs
    pub async fn add_monitored_items(
        &self,
        subscription_id: u32,
        node_ids: &[NodeId],
    ) -> Result<Vec<(NodeId, u32, u32)>> {
        use opcua::types::{MonitoredItemCreateRequest, TimestampsToReturn};

        if node_ids.is_empty() {
            return Ok(Vec::new());
        }

        tracing::info!("Adding {} monitored items to subscription {}", node_ids.len(), subscription_id);

        // Create monitored item requests with unique client handles
        let mut items = Vec::with_capacity(node_ids.len());
        let mut handles = Vec::with_capacity(node_ids.len());

        for node_id in node_ids {
            let client_handle = NEXT_CLIENT_HANDLE.fetch_add(1, Ordering::Relaxed);
            let mut request: MonitoredItemCreateRequest = node_id.clone().into();
            request.requested_parameters.client_handle = client_handle;
            items.push(request);
            handles.push(client_handle);
        }

        // Create the monitored items
        let results = self.session
            .create_monitored_items(subscription_id, TimestampsToReturn::Both, items)
            .await
            .context("Failed to create monitored items")?;

        // Map results to (NodeId, MonitoredItemId, ClientHandle) pairs
        let mut pairs = Vec::new();
        for (i, result) in results.iter().enumerate() {
            if result.result.status_code.is_good() {
                pairs.push((node_ids[i].clone(), result.result.monitored_item_id, handles[i]));
                tracing::debug!("Monitored item created: {:?} -> ID: {}, Handle: {}", node_ids[i], result.result.monitored_item_id, handles[i]);
            } else {
                tracing::warn!("Failed to create monitored item for {:?}: {:?}", node_ids[i], result.result.status_code);
            }
        }

        tracing::info!("Successfully created {} monitored items", pairs.len());
        Ok(pairs)
    }

    /// Remove monitored items from a subscription
    pub async fn remove_monitored_items(
        &self,
        subscription_id: u32,
        item_ids: &[u32],
    ) -> Result<()> {
        if item_ids.is_empty() {
            return Ok(());
        }

        tracing::info!("Removing {} monitored items from subscription {}", item_ids.len(), subscription_id);

        let results = self.session
            .delete_monitored_items(subscription_id, item_ids)
            .await
            .context("Failed to delete monitored items")?;

        for (i, status) in results.iter().enumerate() {
            if !status.is_good() {
                tracing::warn!("Failed to delete monitored item {}: {:?}", item_ids[i], status);
            }
        }

        Ok(())
    }

    /// Delete a subscription
    #[allow(dead_code)]
    pub async fn delete_subscription(&self, subscription_id: u32) -> Result<()> {
        tracing::info!("Deleting subscription {}", subscription_id);
        
        let results = self.session
            .delete_subscriptions(&[subscription_id])
            .await
            .context("Failed to delete subscription")?;

        if let Some(status) = results.first() {
            if !status.is_good() {
                tracing::warn!("Failed to delete subscription {}: {:?}", subscription_id, status);
            }
        }

        Ok(())
    }
}
