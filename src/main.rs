#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! DENGINKS OPC-UA Diagnostic Tool
//!
//! A lightweight, portable, read-only OPC UA Client for diagnostics,
//! structural exporting, and data monitoring.
//!
//! ## Graphics Compatibility
//! This application prioritizes maximum compatibility for Windows Server 2012 R2+
//! by using glow (OpenGL) renderer with automatic fallback to wgpu (DirectX/Vulkan).
//!
//! For systems without hardware graphics acceleration, place `opengl32.dll` from
//! Mesa3D (softpipe) in the same directory as the executable.

mod app;
mod config;
mod export;
mod network;
mod opcua;
mod ui;
mod utils;

use anyhow::Result;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn main() -> Result<()> {
    // Initialize file logging
    let file_appender = tracing_appender::rolling::never(".", "diagnostic.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_writer(non_blocking))
        .with(tracing_subscriber::EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .init();

    tracing::info!("Starting DENGINKS OPC-UA Diagnostic Tool");

    // Install panic hook to log panics
    let next = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        tracing::error!("Application panic: {}", info);
        next(info);
    }));

    // Create tokio runtime for async operations
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    // Check if Mesa3D opengl32.dll exists in the same directory as the executable
    let mesa_dll_exists = check_mesa_dll();
    
    if mesa_dll_exists {
        tracing::info!("Mesa3D opengl32.dll detected - using glow (software OpenGL) renderer");
        // Use glow directly when Mesa3D is available for guaranteed software rendering
        return run_with_renderer(runtime.handle().clone(), eframe::Renderer::Glow);
    }

    // Try renderers in order of compatibility
    // 1. First try wgpu (DirectX 12/Vulkan) - works on modern systems
    // 2. Fallback to glow (OpenGL) - works if OpenGL 2.0+ is available
    
    tracing::info!("Attempting to start with wgpu renderer (DirectX 12 / Vulkan)");
    
    // Configure wgpu for maximum Windows compatibility
    if std::env::var("WGPU_BACKEND").is_err() {
        std::env::set_var("WGPU_BACKEND", "dx12");
    }
    if std::env::var("WGPU_POWER_PREF").is_err() {
        std::env::set_var("WGPU_POWER_PREF", "low");
    }

    let wgpu_result = run_with_renderer(runtime.handle().clone(), eframe::Renderer::Wgpu);
    
    if let Err(wgpu_err) = wgpu_result {
        tracing::warn!("wgpu renderer failed: {}. Trying glow (OpenGL) fallback...", wgpu_err);
        
        // Try glow (OpenGL) fallback
        tracing::info!("Attempting to start with glow renderer (OpenGL)");
        let glow_result = run_with_renderer(runtime.handle().clone(), eframe::Renderer::Glow);
        
        if let Err(glow_err) = glow_result {
            tracing::error!("Both wgpu and glow renderers failed!");
            tracing::error!("wgpu error: {}", wgpu_err);
            tracing::error!("glow error: {}", glow_err);
            
            show_graphics_error(&wgpu_err.to_string(), &glow_err.to_string());
            
            return Err(anyhow::anyhow!(
                "No se pudo inicializar ningún renderizador gráfico. \
                Por favor, descargue opengl32.dll de Mesa3D y colóquelo junto al ejecutable."
            ));
        }
    }

    Ok(())
}

/// Check if Mesa3D opengl32.dll exists in the executable's directory
fn check_mesa_dll() -> bool {
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let mesa_dll = exe_dir.join("opengl32.dll");
            if mesa_dll.exists() {
                // Check if it's likely Mesa3D by checking file size (Mesa is ~36MB)
                if let Ok(metadata) = std::fs::metadata(&mesa_dll) {
                    let size = metadata.len();
                    // Mesa3D opengl32.dll is typically > 30MB, system one is < 1MB
                    if size > 10_000_000 {
                        return true;
                    }
                }
            }
        }
    }
    false
}

/// Run the application with the specified renderer
fn run_with_renderer(runtime_handle: tokio::runtime::Handle, renderer: eframe::Renderer) -> Result<(), anyhow::Error> {
    let renderer_name = match renderer {
        eframe::Renderer::Wgpu => "wgpu",
        eframe::Renderer::Glow => "glow",
    };
    
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("DENGINKS OPC-UA Diagnostic Tool"),
        renderer,
        hardware_acceleration: eframe::HardwareAcceleration::Preferred,
        ..Default::default()
    };

    eframe::run_native(
        "DENGINKS OPC-UA Diagnostic Tool",
        native_options,
        Box::new(move |cc| {
            setup_egui_style(cc);
            tracing::info!("Successfully initialized {} renderer", renderer_name);
            Ok(Box::new(app::DiagnosticApp::new(cc, runtime_handle.clone())))
        }),
    )
    .map_err(|e| anyhow::anyhow!("{}", e))
}

/// Setup egui visual style
fn setup_egui_style(cc: &eframe::CreationContext<'_>) {
    // Initialize egui extras for image loading
    egui_extras::install_image_loaders(&cc.egui_ctx);
    
    // Customize look and feel
    let mut style = (*cc.egui_ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(8.0, 8.0);
    style.spacing.button_padding = egui::vec2(10.0, 6.0);
    
    // Fix for egui 0.30+ API changes (Rounding -> CornerRadius, f32 -> u8)
    use egui::CornerRadius;
    style.visuals.widgets.noninteractive.corner_radius = CornerRadius::same(4);
    style.visuals.widgets.inactive.corner_radius = CornerRadius::same(6);
    style.visuals.widgets.hovered.corner_radius = CornerRadius::same(6);
    style.visuals.widgets.active.corner_radius = CornerRadius::same(6);
    style.visuals.window_corner_radius = CornerRadius::same(10);
    
    cc.egui_ctx.set_style(style);
}

/// Show a native error dialog when graphics initialization fails
fn show_graphics_error(wgpu_err: &str, glow_err: &str) {
    let error_msg = format!(
        "Error de Gráficos - DENGINKS OPC-UA Diagnostic Tool\n\n\
        No se pudo inicializar ningún renderizador gráfico.\n\n\
        Este sistema no tiene soporte para:\n\
        • DirectX 12 / Vulkan (error: {})\n\
        • OpenGL 2.0+ (error: {})\n\n\
        SOLUCIÓN:\n\
        Descargue opengl32.dll de Mesa3D y colóquelo en la\n\
        misma carpeta que el ejecutable.\n\n\
        Mesa3D: https://fdossena.com/?p=mesa/index.fxml\n\
        (Descargar versión x64, extraer opengl32.dll)",
        truncate_error(wgpu_err, 50),
        truncate_error(glow_err, 50)
    );

    #[cfg(target_os = "windows")]
    {
        use std::ffi::CString;
        use std::ptr;
        
        #[link(name = "user32")]
        extern "system" {
            fn MessageBoxA(hwnd: *const (), text: *const i8, caption: *const i8, utype: u32) -> i32;
        }
        
        if let Ok(text) = CString::new(error_msg.clone()) {
            if let Ok(caption) = CString::new("DENGINKS OPC-UA - Error de Gráficos") {
                unsafe {
                    MessageBoxA(ptr::null(), text.as_ptr(), caption.as_ptr(), 0x10); // MB_ICONERROR
                }
            }
        }
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        eprintln!("{}", error_msg);
    }
}

/// Truncate error message for display
fn truncate_error(err: &str, max_len: usize) -> String {
    if err.len() > max_len {
        format!("{}...", &err[..max_len])
    } else {
        err.to_string()
    }
}
