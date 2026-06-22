use std::sync::Arc;
use parking_lot::Mutex;
use typshow::AppState;
use typshow::TypshowApp;
use tracing::{info, error};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"))
        )
        .init();

    info!("Iniciando aplicación Typshow...");

    let args: Vec<String> = std::env::args().collect();
    let mut state = AppState::new();

    if args.len() >= 2 {
        let file_path = &args[1];
        info!("Cargando documento inicial desde argumento: {}", file_path);
        if let Err(e) = state.load_file(file_path) {
            error!("Error cargando documento inicial '{}': {:?}", file_path, e);
        }
    } else {
        info!("No se especificó un documento inicial. Iniciando en modo vacío.");
    }

    let shared_state = Arc::new(Mutex::new(state));

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Typshow - Controlador")
            .with_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    info!("Iniciando ventana de Typshow...");
    eframe::run_native(
        "Typshow",
        options,
        Box::new(|_cc| {
            Ok(Box::new(TypshowApp::new(shared_state)))
        }),
    )?;

    info!("Aplicación terminada con éxito.");
    Ok(())
}