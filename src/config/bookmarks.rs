//! Server bookmarks save/load functionality

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::utils::i18n::{self, T, Language};

/// Security policy for OPC-UA connection
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum SecurityPolicy {
    #[default]
    None,
    Basic128Rsa15,
    Basic256,
    Basic256Sha256,
    Aes128Sha256RsaOaep,
    Aes256Sha256RsaPss,
}

impl SecurityPolicy {
    /// Get all available security policies
    pub fn all() -> Vec<Self> {
        vec![
            Self::None,
            Self::Basic128Rsa15,
            Self::Basic256,
            Self::Basic256Sha256,
            Self::Aes128Sha256RsaOaep,
            Self::Aes256Sha256RsaPss,
        ]
    }

    /// Convert to display string
    pub fn display_name(&self, lang: Language) -> String {
        match self {
            Self::None => i18n::t(T::SecurityNone, lang).to_string(),
            Self::Basic128Rsa15 => i18n::t(T::SecurityBasic128Rsa15, lang).to_string(),
            Self::Basic256 => i18n::t(T::SecurityBasic256, lang).to_string(),
            Self::Basic256Sha256 => i18n::t(T::SecurityBasic256Sha256, lang).to_string(),
            Self::Aes128Sha256RsaOaep => i18n::t(T::SecurityAes128Sha256RsaOaep, lang).to_string(),
            Self::Aes256Sha256RsaPss => i18n::t(T::SecurityAes256Sha256RsaPss, lang).to_string(),
        }
    }
}

/// Message security mode
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum MessageSecurityMode {
    #[default]
    None,
    Sign,
    SignAndEncrypt,
}

impl MessageSecurityMode {
    /// Get all available modes
    pub fn all() -> Vec<Self> {
        vec![Self::None, Self::Sign, Self::SignAndEncrypt]
    }

    /// Convert to display string
    pub fn display_name(&self, lang: Language) -> String {
        match self {
            Self::None => i18n::t(T::ModeNone, lang).to_string(),
            Self::Sign => i18n::t(T::ModeSign, lang).to_string(),
            Self::SignAndEncrypt => i18n::t(T::ModeSignAndEncrypt, lang).to_string(),
        }
    }
}

/// Authentication method
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum AuthMethod {
    #[default]
    Anonymous,
    UserPassword {
        username: String,
        password: String,
    },
}

impl AuthMethod {}

/// A saved server bookmark
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServerBookmark {
    /// Display name for the bookmark
    pub name: String,
    /// OPC-UA endpoint URL (e.g., opc.tcp://localhost:4840)
    pub endpoint_url: String,
    /// Security policy
    pub security_policy: SecurityPolicy,
    /// Message security mode
    pub security_mode: MessageSecurityMode,
    /// Authentication method
    pub auth_method: AuthMethod,
}

impl ServerBookmark {}

/// Collection of server bookmarks
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Bookmarks {
    /// List of saved bookmarks
    pub servers: Vec<ServerBookmark>,
}

impl Bookmarks {
    /// Get the path to the bookmarks file
    fn bookmarks_path() -> PathBuf {
        // Store next to the executable
        std::env::current_exe()
            .unwrap_or_else(|_| PathBuf::from("."))
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."))
            .join("bookmarks.json")
    }

    /// Load bookmarks from file
    pub fn load() -> Result<Self> {
        let path = Self::bookmarks_path();
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            let bookmarks: Bookmarks = serde_json::from_str(&content)?;
            tracing::info!("Loaded {} bookmarks from {:?}", bookmarks.servers.len(), path);
            Ok(bookmarks)
        } else {
            tracing::info!("No bookmarks file found, starting fresh");
            Ok(Self::default())
        }
    }

    /// Save bookmarks to file
    pub fn save(&self) -> Result<()> {
        let path = Self::bookmarks_path();
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        tracing::info!("Saved {} bookmarks to {:?}", self.servers.len(), path);
        Ok(())
    }

    /// Add a new bookmark
    pub fn add(&mut self, bookmark: ServerBookmark) {
        self.servers.push(bookmark);
    }

    /// Remove a bookmark by index
    pub fn remove(&mut self, index: usize) {
        if index < self.servers.len() {
            self.servers.remove(index);
        }
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.servers.is_empty()
    }
}
