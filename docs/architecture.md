# Architecture Guide

This document describes the high-level architecture of the DENGINKS OPC-UA Diagnostic Tool.

## Overview

The application is built using **Rust** and follows a modular architecture separating the User Interface (UI), Business Logic, and Network Communication layers.

### Core Technologies

*   **Runtime:** `tokio` (Async runtime for non-blocking I/O)
*   **GUI Framework:** `egui` + `eframe` (Immediate mode GUI)
*   **OPC-UA Client:** `async-opcua` (Asynchronous OPC-UA implementation)
*   **Graphics Backend:** Supports both `wgpu` (DirectX12/Vulkan) and `glow` (OpenGL) with automatic fallback.

## Directory Structure

The project was refactored from a binary crate to a library-based structure to support integration testing.

*   `src/lib.rs`: The library entry point, exposing modules.
*   `src/main.rs`: The binary entry point. Handles renderer selection and runtime initialization.
*   `src/app.rs`: The main application state machine and UI loop.
*   `src/opcua/`: Contains all OPC-UA related logic (Client, Subscription, Certificates).
*   `src/network/`: Networking utilities, diagnostics, and pre-checks.
*   `src/ui/`: UI components broken down by functional panels.

## Key Subsystems

### 1. Application Loop (`src/app.rs`)
The `DiagnosticApp` struct holds the global state. It communicates with the async OPC-UA client using `tokio` channels (`mpsc`). The `update()` method is called every frame by `eframe` to render the UI.

### 2. OPC-UA Client (`src/opcua/client.rs`)
Runs in a separate `tokio` task. It manages:
*   Session connection/disconnection.
*   Subscription management (creating/deleting monitored items).
*   Browsing the address space.

### 3. Subscription Management (`src/opcua/subscription.rs`)
Handles data monitoring. It maps `NodeId`s to internal handles and manages a buffer of historical values (`MonitoredData`) for trending charts.

### 4. Networking Diagnostics (`src/network/diagnostics.rs`)
Before connecting, the application performs a "Pre-check":
1.  **Parsing:** Validates the URL format.
2.  **DNS Resolution:** Resolves the hostname to an IP.
3.  **Port Scanning:** Checks if TCP ports (4840, etc.) are open.
4.  **Endpoint Discovery:** Queries the server for supported security policies.

## Testing Strategy

*   **Unit Tests:** Located within source files (e.g., `src/config/settings.rs`), testing isolated logic.
*   **Integration Tests:** Located in `tests/`, checking end-to-end flows (e.g., `network_integration.rs` spawns a TCP listener to verify port scanning).
