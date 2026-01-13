




use std::net::ToSocketAddrs;
use std::time::{Duration, Instant};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::time::timeout;
use tokio_util::sync::CancellationToken;

use crate::network::discovery;
use crate::utils::i18n::{self, t, T, Language};


pub const OPCUA_COMMON_PORTS: &[u16] = &[4840, 4841, 4842, 4843, 48010, 48020, 62541];


#[derive(Debug, Clone, PartialEq)]
pub enum StepStatus {
    Pending,
    Running,
    Success,
    Warning,
    Failed,
}

impl StepStatus {
    
    pub fn icon(&self) -> &'static str {
        match self {
            StepStatus::Pending => "⏳",
            StepStatus::Running => "🔄",
            StepStatus::Success => "✅",
            StepStatus::Warning => "⚠️",
            StepStatus::Failed => "❌",
        }
    }
}

/// A single diagnostic step result
#[derive(Debug, Clone)]
pub struct DiagnosticStep {
    /// Step identifier
    pub id: StepId,
    /// Human-readable step name
    pub name: String,
    /// Current status
    pub status: StepStatus,
    /// Detailed description or result
    pub details: String,
    /// Time taken in milliseconds
    pub duration_ms: u64,
}

impl DiagnosticStep {
    pub fn new(id: StepId, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            status: StepStatus::Pending,
            details: String::new(),
            duration_ms: 0,
        }
    }

    pub fn running(mut self, details: impl Into<String>) -> Self {
        self.status = StepStatus::Running;
        self.details = details.into();
        self
    }

    pub fn success(mut self, details: impl Into<String>, duration_ms: u64) -> Self {
        self.status = StepStatus::Success;
        self.details = details.into();
        self.duration_ms = duration_ms;
        self
    }

    pub fn warning(mut self, details: impl Into<String>, duration_ms: u64) -> Self {
        self.status = StepStatus::Warning;
        self.details = details.into();
        self.duration_ms = duration_ms;
        self
    }

    pub fn failed(mut self, details: impl Into<String>, duration_ms: u64) -> Self {
        self.status = StepStatus::Failed;
        self.details = details.into();
        self.duration_ms = duration_ms;
        self
    }
}

/// Step identifiers for tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepId {
    ValidateInput,
    ResolveDns,
    ScanPorts,
    DiscoverEndpoints,
}

/// Parsed user input
#[derive(Debug, Clone)]
pub struct ParsedInput {
    /// Extracted host (IP or hostname)
    pub host: String,
    /// Extracted port (if any)
    pub port: Option<u16>,
    /// Whether input had opc.tcp:// scheme
    pub had_scheme: bool,
    /// Validation errors
    pub errors: Vec<String>,
}

impl ParsedInput {
    /// Check if input is valid
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty() && !self.host.is_empty()
    }

    /// Build URL with specified port
    pub fn to_url(&self, port: u16) -> String {
        format!("opc.tcp://{}:{}", self.host, port)
    }
}

/// Result of a port scan
#[derive(Debug, Clone)]
pub struct PortScanResult {
    pub port: u16,
    pub open: bool,
}

/// Complete diagnostic result
#[derive(Debug, Clone)]
pub struct DiagnosticResult {
    /// All diagnostic steps
    pub steps: Vec<DiagnosticStep>,
    /// Overall success
    pub overall_success: bool,
    /// Discovered open ports
    pub open_ports: Vec<PortScanResult>,
    /// Recommended URL to use
    pub recommended_url: Option<String>,
    /// Discovered endpoints (if any)
    pub endpoints: Vec<discovery::EndpointInfo>,
    /// Total time taken
    pub total_duration_ms: u64,
}

impl DiagnosticResult {
    pub fn new() -> Self {
        Self {
            steps: Vec::new(),
            overall_success: false,
            open_ports: Vec::new(),
            recommended_url: None,
            endpoints: Vec::new(),
            total_duration_ms: 0,
        }
    }
}

impl Default for DiagnosticResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse user input into structured data
///
/// Accepts:
/// - IP address: `192.168.1.100`
/// - Hostname: `myserver.local`
/// - IP with port: `192.168.1.100:4840`
/// - Full URL: `opc.tcp://192.168.1.100:4840/Path`
pub fn parse_user_input(input: &str) -> ParsedInput {
    let trimmed = input.trim();
    let mut result = ParsedInput {
        host: String::new(),
        port: None,
        had_scheme: false,
        errors: Vec::new(),
    };

    if trimmed.is_empty() {
        result.errors.push("Input cannot be empty".to_string());
        return result;
    }

    // Check for scheme
    let without_scheme = if let Some(rest) = trimmed.strip_prefix("opc.tcp://") {
        result.had_scheme = true;
        rest
    } else if trimmed.contains("://") {
        result.errors.push("Only opc.tcp:// scheme is supported".to_string());
        return result;
    } else {
        trimmed
    };

    // Remove path if present
    let host_port = without_scheme.split('/').next().unwrap_or(without_scheme);

    // Handle IPv6 addresses
    if host_port.starts_with('[') {
        // IPv6 format: [::1]:port
        if let Some(bracket_end) = host_port.find(']') {
            result.host = host_port[..=bracket_end].to_string();
            let after_bracket = &host_port[bracket_end + 1..];
            if let Some(port_str) = after_bracket.strip_prefix(':') {
                match port_str.parse::<u16>() {
                    Ok(p) => result.port = Some(p),
                    Err(_) => result.errors.push(format!("Invalid port: {}", port_str)),
                }
            }
        } else {
            result.errors.push("Invalid IPv6 address format".to_string());
        }
    } else {
        // IPv4 or hostname
        let parts: Vec<&str> = host_port.rsplitn(2, ':').collect();
        match parts.len() {
            1 => {
                result.host = parts[0].to_string();
            }
            2 => {
                
                if let Ok(p) = parts[0].parse::<u16>() {
                    result.port = Some(p);
                    result.host = parts[1].to_string();
                } else {
                    
                    result.host = host_port.to_string();
                }
            }
            _ => {
                result.errors.push("Invalid host:port format".to_string());
            }
        }
    }

    
    if result.host.is_empty() {
        result.errors.push("Host cannot be empty".to_string());
    }

    result
}


pub async fn run_diagnostic(
    input: &str,
    progress_tx: mpsc::Sender<DiagnosticStep>,
    cancel: CancellationToken,
    lang: Language,
) -> DiagnosticResult {
    let start = Instant::now();
    let mut result = DiagnosticResult::new();

    
    let step1 = DiagnosticStep::new(StepId::ValidateInput, t(T::ValidatingUrl, lang));
    let _ = progress_tx.send(step1.clone().running(t(T::ValidatingUrl, lang))).await;

    let parsed = parse_user_input(input);
    
    if !parsed.is_valid() {
        let step = step1.failed(parsed.errors.join(", "), 0);
        let _ = progress_tx.send(step.clone()).await;
        result.steps.push(step);
        result.total_duration_ms = start.elapsed().as_millis() as u64;
        return result;
    }

    let step = step1.success(format!("Host: {}, Port: {:?}", parsed.host, parsed.port), 0);
    let _ = progress_tx.send(step.clone()).await;
    result.steps.push(step);

    
    if cancel.is_cancelled() {
        result.total_duration_ms = start.elapsed().as_millis() as u64;
        return result;
    }

    
    let step2 = DiagnosticStep::new(StepId::ResolveDns, t(T::ResolvingDns, lang));
    let _ = progress_tx.send(step2.clone().running(format!("Resolving {}...", parsed.host))).await;

    let dns_start = Instant::now();
    let addr_result = format!("{}:4840", parsed.host).to_socket_addrs();
    let dns_duration = dns_start.elapsed().as_millis() as u64;

    let resolved_ip = match addr_result {
        Ok(mut addrs) => {
            if let Some(addr) = addrs.next() {
                let ip = addr.ip().to_string();
                let step = step2.success(format!("{} → {}", parsed.host, ip), dns_duration);
                let _ = progress_tx.send(step.clone()).await;
                result.steps.push(step);
                Some(ip)
            } else {
                let step = step2.failed(t(T::DnsFailed, lang), dns_duration);
                let _ = progress_tx.send(step.clone()).await;
                result.steps.push(step);
                result.total_duration_ms = start.elapsed().as_millis() as u64;
                return result;
            }
        }
        Err(e) => {
            let step = step2.failed(format!("{}: {}", t(T::DnsFailed, lang), e), dns_duration);
            let _ = progress_tx.send(step.clone()).await;
            result.steps.push(step);
            result.total_duration_ms = start.elapsed().as_millis() as u64;
            return result;
        }
    };

    
    if cancel.is_cancelled() {
        result.total_duration_ms = start.elapsed().as_millis() as u64;
        return result;
    }

    
    let step3 = DiagnosticStep::new(StepId::ScanPorts, t(T::ScanningPorts, lang));
    
    
    let ports_to_scan: Vec<u16> = if let Some(p) = parsed.port {
        vec![p]
    } else {
        OPCUA_COMMON_PORTS.to_vec()
    };

    let _ = progress_tx.send(step3.clone().running(format!(
        "{}: {:?}",
        t(T::ScanningPorts, lang),
        ports_to_scan
    ))).await;

    let scan_start = Instant::now();
    let host = resolved_ip.as_ref().unwrap_or(&parsed.host);
    
    for port in &ports_to_scan {
        if cancel.is_cancelled() {
            break;
        }

        let addr = format!("{}:{}", host, port);
        
        let open = matches!(timeout(Duration::from_secs(2), TcpStream::connect(&addr)).await, Ok(Ok(_)));

        result.open_ports.push(PortScanResult {
            port: *port,
            open,
        });
    }

    let scan_duration = scan_start.elapsed().as_millis() as u64;
    let open_count = result.open_ports.iter().filter(|p| p.open).count();

    if open_count == 0 {
        let step = step3.failed(
            format!("{} (tested: {:?})", t(T::NoOpenPorts, lang), ports_to_scan),
            scan_duration,
        );
        let _ = progress_tx.send(step.clone()).await;
        result.steps.push(step);
        result.total_duration_ms = start.elapsed().as_millis() as u64;
        return result;
    }

    let open_ports_str: Vec<String> = result.open_ports.iter()
        .filter(|p| p.open)
        .map(|p| p.port.to_string())
        .collect();

    let step = step3.success(
        format!("{}: {}", t(T::PortsOpen, lang), open_ports_str.join(", ")),
        scan_duration,
    );
    let _ = progress_tx.send(step.clone()).await;
    result.steps.push(step);

    
    if cancel.is_cancelled() {
        result.total_duration_ms = start.elapsed().as_millis() as u64;
        return result;
    }

    
    let step4 = DiagnosticStep::new(StepId::DiscoverEndpoints, t(T::DiscoveringEndpoints, lang));
    let _ = progress_tx.send(step4.clone().running(t(T::DiscoveringEndpoints, lang))).await;

    let discovery_start = Instant::now();
    
    for port_result in result.open_ports.iter().filter(|p| p.open) {
        if cancel.is_cancelled() {
            break;
        }

        let url = parsed.to_url(port_result.port);
        
        match discovery::discover_endpoints(&url).await {
            Ok(endpoints) if !endpoints.is_empty() => {
                let recommended_url = endpoints[0].endpoint_url.clone();
                result.endpoints = endpoints;
                result.recommended_url = Some(recommended_url);
                result.overall_success = true;
                break;
            }
            _ => continue,
        }
    }

    let discovery_duration = discovery_start.elapsed().as_millis() as u64;

    if result.overall_success {
        let step = step4.success(
            format!("{} endpoints found at {}", 
                result.endpoints.len(),
                result.recommended_url.as_ref().unwrap_or(&String::new())
            ),
            discovery_duration,
        );
        let _ = progress_tx.send(step.clone()).await;
        result.steps.push(step);
    } else {
        let step = step4.warning(
            i18n::t(T::NoEndpointsFound, lang).to_string(),
            discovery_duration,
        );
        let _ = progress_tx.send(step.clone()).await;
        result.steps.push(step);
    }

    result.total_duration_ms = start.elapsed().as_millis() as u64;
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ip_only() {
        let result = parse_user_input("192.168.1.100");
        assert!(result.is_valid());
        assert_eq!(result.host, "192.168.1.100");
        assert_eq!(result.port, None);
        assert!(!result.had_scheme);
    }

    #[test]
    fn test_parse_ip_with_port() {
        let result = parse_user_input("192.168.1.100:4840");
        assert!(result.is_valid());
        assert_eq!(result.host, "192.168.1.100");
        assert_eq!(result.port, Some(4840));
    }

    #[test]
    fn test_parse_full_url() {
        let result = parse_user_input("opc.tcp://myserver.local:4840/UA/Server");
        assert!(result.is_valid());
        assert_eq!(result.host, "myserver.local");
        assert_eq!(result.port, Some(4840));
        assert!(result.had_scheme);
    }

    #[test]
    fn test_parse_hostname_only() {
        let result = parse_user_input("myserver.local");
        assert!(result.is_valid());
        assert_eq!(result.host, "myserver.local");
        assert_eq!(result.port, None);
    }

    #[test]
    fn test_parse_invalid_scheme() {
        let result = parse_user_input("http://192.168.1.100:4840");
        assert!(!result.is_valid());
        assert!(result.errors[0].contains("opc.tcp://"));
    }

    #[test]
    fn test_parse_empty() {
        let result = parse_user_input("");
        assert!(!result.is_valid());
    }
}
