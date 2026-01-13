




use opcua::client::ClientBuilder;
use opcua::types::MessageSecurityMode as OpcMessageSecurityMode;
use crate::utils::i18n::{self, T, Language};


#[derive(Debug, Clone)]
pub struct EndpointInfo {
    
    pub security_policy_name: String,
    
    pub security_mode: String,
    
    pub has_certificate: bool,
    
    pub user_tokens: Vec<String>,
    
    pub endpoint_url: String,
}

impl EndpointInfo {
    
    pub fn allows_anonymous(&self) -> bool {
        self.user_tokens.iter().any(|t| t.to_lowercase().contains("anonymous"))
    }
}


pub async fn discover_endpoints(discovery_url: &str) -> Result<Vec<EndpointInfo>, String> {
    tracing::info!("Discovering endpoints at {}", discovery_url);
    
    
    let client = ClientBuilder::new()
        .application_name("DengInks OPC-UA Discovery")
        .application_uri("urn:DengInks:OpcUaDiagnostic:Discovery")
        .client()
        .map_err(|e| format!("Failed to create discovery client: {:?}", e))?;

    
    let endpoints = client
        .get_server_endpoints_from_url(discovery_url)
        .await
        .map_err(|e| format!("Failed to get endpoints: {}", e))?;

    if endpoints.is_empty() {
        return Err("No endpoints returned from server".to_string());
    }

    tracing::info!("Discovered {} endpoints", endpoints.len());

    
    let endpoint_infos: Vec<EndpointInfo> = endpoints
        .into_iter()
        .map(|ep| {
            
            let policy_uri = ep.security_policy_uri.as_ref().to_string();
            let policy_name = parse_security_policy_name(&policy_uri);

            
            let mode_str = match ep.security_mode {
                OpcMessageSecurityMode::None => "None",
                OpcMessageSecurityMode::Sign => "Sign",
                OpcMessageSecurityMode::SignAndEncrypt => "SignAndEncrypt",
                _ => "Unknown",
            };

            
            let user_tokens: Vec<String> = ep
                .user_identity_tokens
                .as_ref()
                .map(|tokens| {
                    tokens
                        .iter()
                        .map(|t| {
                            let policy_id = t.policy_id.as_ref().to_string();
                            let token_type = match t.token_type {
                                opcua::types::UserTokenType::Anonymous => "Anonymous",
                                opcua::types::UserTokenType::UserName => "UserName",
                                opcua::types::UserTokenType::Certificate => "Certificate",
                                opcua::types::UserTokenType::IssuedToken => "IssuedToken",
                            };
                            format!("{} ({})", token_type, policy_id)
                        })
                        .collect()
                })
                .unwrap_or_default();

            
            let has_certificate = !ep.server_certificate.is_null();

            EndpointInfo {
                security_policy_name: policy_name,
                security_mode: mode_str.to_string(),
                has_certificate,
                user_tokens,
                endpoint_url: ep.endpoint_url.as_ref().to_string(),
            }
        })
        .collect();

    Ok(endpoint_infos)
}


fn parse_security_policy_name(uri: &str) -> String {
    
    if let Some(hash_pos) = uri.rfind('#') {
        uri[hash_pos + 1..].to_string()
    } else if let Some(slash_pos) = uri.rfind('/') {
        uri[slash_pos + 1..].to_string()
    } else if uri.is_empty() || uri.to_lowercase().contains("none") {
        "None".to_string()
    } else {
        uri.to_string()
    }
}

impl EndpointInfo {
    
    pub fn display_name(&self, lang: Language) -> String {
        let cert_icon = if self.has_certificate { "🔐" } else { "⚠️" };
        let auth_str = if self.allows_anonymous() {
            i18n::t(T::Anonymous, lang)
        } else {
            i18n::t(T::AuthRequired, lang)
        };
        
        format!(
            "{} {} - {} ({})",
            cert_icon,
            self.security_policy_name,
            self.security_mode,
            auth_str
        )
    }
}

impl std::fmt::Display for EndpointInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} - {} {}",
            if self.has_certificate { "🔐" } else { "⚠️" },
            self.security_policy_name,
            self.security_mode,
            if self.allows_anonymous() { "(Anonymous)" } else { "(Auth Required)" }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_security_policy() {
        assert_eq!(
            parse_security_policy_name("http://opcfoundation.org/UA/SecurityPolicy#Basic256Sha256"),
            "Basic256Sha256"
        );
        assert_eq!(
            parse_security_policy_name("http://opcfoundation.org/UA/SecurityPolicy#None"),
            "None"
        );
        assert_eq!(
            parse_security_policy_name(""),
            "None"
        );
    }
}
