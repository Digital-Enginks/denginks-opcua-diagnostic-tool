

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::utils::i18n::{self, T, Language};


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


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum MessageSecurityMode {
    #[default]
    None,
    Sign,
    SignAndEncrypt,
}

impl MessageSecurityMode {
    
    pub fn all() -> Vec<Self> {
        vec![Self::None, Self::Sign, Self::SignAndEncrypt]
    }

    
    pub fn display_name(&self, lang: Language) -> String {
        match self {
            Self::None => i18n::t(T::ModeNone, lang).to_string(),
            Self::Sign => i18n::t(T::ModeSign, lang).to_string(),
            Self::SignAndEncrypt => i18n::t(T::ModeSignAndEncrypt, lang).to_string(),
        }
    }
}


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


#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServerBookmark {
    
    pub name: String,
    
    pub endpoint_url: String,
    
    pub security_policy: SecurityPolicy,
    
    pub security_mode: MessageSecurityMode,
    
    pub auth_method: AuthMethod,
}

impl ServerBookmark {}


#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Bookmarks {
    
    pub servers: Vec<ServerBookmark>,
}

impl Bookmarks {
    
    fn bookmarks_path() -> PathBuf {
        
        std::env::current_exe()
            .unwrap_or_else(|_| PathBuf::from("."))
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."))
            .join("bookmarks.json")
    }

    
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

    
    pub fn save(&self) -> Result<()> {
        let path = Self::bookmarks_path();
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        tracing::info!("Saved {} bookmarks to {:?}", self.servers.len(), path);
        Ok(())
    }

    
    pub fn add(&mut self, bookmark: ServerBookmark) {
        self.servers.push(bookmark);
    }

    
    pub fn remove(&mut self, index: usize) {
        if index < self.servers.len() {
            self.servers.remove(index);
        }
    }

    
    pub fn is_empty(&self) -> bool {
        self.servers.is_empty()
    }
}
