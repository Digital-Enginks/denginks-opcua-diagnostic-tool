//! Network pre-check functionality
//! 
//! Performs URL parsing and TCP connectivity checks for OPC-UA connections.

/// Parse an OPC-UA endpoint URL to extract host and port
/// 
/// Supports URLs like:
/// - opc.tcp://localhost:4840
/// - opc.tcp://192.168.1.100:4840/path
pub fn parse_endpoint_url(url: &str) -> Result<(String, u16), String> {
    // Remove the opc.tcp:// prefix
    let without_scheme = url
        .strip_prefix("opc.tcp://")
        .ok_or_else(|| "URL must start with opc.tcp://".to_string())?;

    // Split off any path
    let host_port = without_scheme
        .split('/')
        .next()
        .ok_or_else(|| "Invalid URL format".to_string())?;

    // Split host and port
    let parts: Vec<&str> = host_port.rsplitn(2, ':').collect();
    
    match parts.len() {
        2 => {
            let port = parts[0]
                .parse::<u16>()
                .map_err(|_| format!("Invalid port: {}", parts[0]))?;
            let host = parts[1].to_string();
            Ok((host, port))
        }
        1 => {
            let host = parts[0].to_string();
            if host.is_empty() {
                return Err("Host cannot be empty".to_string());
            }
            // Default port for OPC-UA
            Ok((host, 4840))
        }
        _ => Err("Invalid host:port format".to_string()),
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_endpoint_url() {
        assert_eq!(
            parse_endpoint_url("opc.tcp://localhost:4840").unwrap(),
            ("localhost".to_string(), 4840)
        );
        
        assert_eq!(
            parse_endpoint_url("opc.tcp://192.168.1.100:4841/path").unwrap(),
            ("192.168.1.100".to_string(), 4841)
        );
        
        assert_eq!(
            parse_endpoint_url("opc.tcp://server.example.com").unwrap(),
            ("server.example.com".to_string(), 4840)
        );

        // IPv6 (basic support)
        assert_eq!(
            parse_endpoint_url("opc.tcp://[::1]:4840").unwrap(),
            ("[::1]".to_string(), 4840)
        );
        
        // Malformed URLs
        assert!(parse_endpoint_url("http://localhost:4840").is_err());
        assert!(parse_endpoint_url("opc.tcp://").is_err());
        assert!(parse_endpoint_url("opc.tcp://host:port:extra").is_err());
    }
}
