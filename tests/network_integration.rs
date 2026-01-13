use denginks_opcua_diagnostic::network::diagnostics::{self, StepId, run_diagnostic};
use denginks_opcua_diagnostic::utils::i18n::Language;
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

#[tokio::test]
async fn test_port_scan_success() {
    // 1. Start a dummy TCP listener
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("Failed to bind");
    let local_addr = listener.local_addr().expect("Failed to get addr");
    let port = local_addr.port();

    // Spawn a task to accept the connection so the scan succeeds
    tokio::spawn(async move {
        if let Ok(_) = listener.accept().await {
            // Just accept and close
        }
    });

    // 2. Run diagnostic
    let input = format!("127.0.0.1:{}", port);
    let (tx, mut rx) = mpsc::channel(100);
    let cancel = CancellationToken::new();
    
    // Spawn a consumer for the progress channel to prevent blocking
    tokio::spawn(async move {
        while let Some(_) = rx.recv().await {}
    });

    let result = run_diagnostic(&input, tx, cancel, Language::English).await;

    // 3. Verify
    // Check if the port was found open
    let found = result.open_ports.iter().any(|p| p.port == port && p.open);
    assert!(found, "Should have found open port {}", port);
    
    // Check steps for success
    let scan_step = result.steps.iter().find(|s| s.id == StepId::ScanPorts).expect("ScanPorts step missing");
    assert_eq!(scan_step.status, diagnostics::StepStatus::Success);
}

#[tokio::test]
async fn test_port_scan_fail() {
    // Pick a port likely closed (not 100% robust but usually fine for local test)
    // Better: bind and close?
    // If I bind port 0, get port X, then drop listener, port X is likely closed immediately after.
    
    let port;
    {
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("Failed to bind");
        port = listener.local_addr().expect("Failed to get addr").port();
    } // listener dropped, port closed

    let input = format!("127.0.0.1:{}", port);
    let (tx, mut rx) = mpsc::channel(100);
    let cancel = CancellationToken::new();
    
    tokio::spawn(async move {
        while let Some(_) = rx.recv().await {}
    });

    let result = run_diagnostic(&input, tx, cancel, Language::English).await;

    let found_open = result.open_ports.iter().any(|p| p.port == port && p.open);
    assert!(!found_open, "Port {} should be closed", port);
}
