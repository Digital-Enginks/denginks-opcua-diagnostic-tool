//! Certificate management for OPC-UA connections
//!
//! Handles PKI directory structure and certificate trust management.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::fs;

/// Certificate manager for OPC-UA PKI operations
pub struct CertificateManager {
    /// Root PKI directory (next to executable)
    pki_dir: PathBuf,
    /// Path to trusted server certificates directory
    trusted_certs_dir: PathBuf,
    /// Path to rejected certificates directory
    rejected_certs_dir: PathBuf,
}

impl CertificateManager {
    /// Create a new certificate manager with PKI directory next to executable
    pub fn new() -> Result<Self> {
        let exe_dir = std::env::current_exe()
            .context("Failed to get executable path")?
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."));

        let pki_dir = exe_dir.join("pki");
        let trusted_certs_dir = pki_dir.join("trusted").join("certs");
        let rejected_certs_dir = pki_dir.join("rejected").join("certs");

        Ok(Self {
            pki_dir,
            trusted_certs_dir,
            rejected_certs_dir,
        })
    }

    /// Get the PKI directory path
    pub fn pki_directory(&self) -> &Path {
        &self.pki_dir
    }


    /// Ensure PKI directory structure exists
    pub fn ensure_pki_structure(&self) -> Result<()> {
        // Create directory structure:
        // pki/
        //   own/           - Client certificate and key (created by async-opcua)
        //   trusted/certs/ - Trusted server certificates  
        //   rejected/certs/ - Rejected certificates
        
        let dirs = [
            self.pki_dir.join("own"),
            self.pki_dir.join("private"),
            self.trusted_certs_dir.clone(),
            self.rejected_certs_dir.clone(),
        ];

        for dir in &dirs {
            if !dir.exists() {
                fs::create_dir_all(dir)
                    .with_context(|| format!("Failed to create directory: {:?}", dir))?;
                tracing::info!("Created PKI directory: {:?}", dir);
            }
        }

        Ok(())
    }

    /// List certificate files in a directory
    fn list_certs_in_dir(dir: &Path) -> Vec<CertificateInfo> {
        if !dir.exists() {
            return Vec::new();
        }

        fs::read_dir(dir)
            .ok()
            .map(|entries| {
                entries
                    .filter_map(|entry| entry.ok())
                    .filter(|entry| {
                        let path = entry.path();
                        path.is_file() && matches!(
                            path.extension().and_then(|e| e.to_str()),
                            Some("der") | Some("crt") | Some("pem")
                        )
                    })
                    .map(|entry| CertificateInfo {
                        path: entry.path(),
                        name: entry.file_name().to_string_lossy().to_string(),
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// List trusted server certificates
    pub fn list_trusted_certs(&self) -> Vec<CertificateInfo> {
        Self::list_certs_in_dir(&self.trusted_certs_dir)
    }

    /// List rejected certificates
    pub fn list_rejected_certs(&self) -> Vec<CertificateInfo> {
        Self::list_certs_in_dir(&self.rejected_certs_dir)
    }

    /// Get client certificate path if it exists
    pub fn get_client_cert(&self) -> Option<CertificateInfo> {
        let own_dir = self.pki_dir.join("own");
        if own_dir.exists() {
            // Look for cert.der or similar
            for name in ["cert.der", "client.der", "cert.pem", "client.pem"] {
                let path = own_dir.join(name);
                if path.exists() {
                    return Some(CertificateInfo {
                        path: path.clone(),
                        name: name.to_string(),
                    });
                }
            }
        }
        None
    }

    /// Trust a rejected certificate (move from rejected to trusted)
    pub fn trust_certificate(&self, cert_path: &Path) -> Result<()> {
        if !cert_path.exists() {
            anyhow::bail!("Certificate file not found: {:?}", cert_path);
        }

        let file_name = cert_path.file_name()
            .context("Invalid certificate path")?;
        
        let dest = self.trusted_certs_dir.join(file_name);
        fs::rename(cert_path, &dest)
            .with_context(|| format!("Failed to move certificate to trusted: {:?}", dest))?;
        
        tracing::info!("Trusted certificate: {:?}", file_name);
        Ok(())
    }

    /// Delete a certificate file
    pub fn delete_certificate(&self, cert_path: &Path) -> Result<()> {
        if !cert_path.exists() {
            anyhow::bail!("Certificate file not found: {:?}", cert_path);
        }

        fs::remove_file(cert_path)
            .with_context(|| format!("Failed to delete certificate: {:?}", cert_path))?;
        
        tracing::info!("Deleted certificate: {:?}", cert_path);
        Ok(())
    }

    /// Open the PKI folder in the system file explorer
    #[cfg(target_os = "windows")]
    pub fn open_pki_folder(&self) -> Result<()> {
        std::process::Command::new("explorer")
            .arg(&self.pki_dir)
            .spawn()
            .context("Failed to open PKI folder")?;
        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    pub fn open_pki_folder(&self) -> Result<()> {
        std::process::Command::new("xdg-open")
            .arg(&self.pki_dir)
            .spawn()
            .context("Failed to open PKI folder")?;
        Ok(())
    }
}

/// Certificate info for UI display
#[derive(Debug, Clone)]
pub struct CertificateInfo {
    /// Full path to the certificate file
    pub path: PathBuf,
    /// File name
    pub name: String,
}

impl Default for CertificateManager {
    fn default() -> Self {
        Self::new().expect("Failed to create CertificateManager")
    }
}

