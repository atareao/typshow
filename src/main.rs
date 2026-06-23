use std::sync::Arc;
use parking_lot::Mutex;
use typshow::AppState;
use typshow::TypshowApp;
use tracing::{info, error};

fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    let mut db = fontdb::Database::new();
    db.load_system_fonts();

    let mut loaded_emoji = 0;
    let mut has_cantarell = false;

    for face in db.faces() {
        let families: Vec<String> = face.families.iter().map(|(n, _)| n.to_lowercase()).collect();

        let is_cantarell = families.iter().any(|n| n == "cantarell");
        if is_cantarell {
            if let fontdb::Source::File(path) = &face.source {
                if let Ok(data) = std::fs::read(path) {
                    let font_name = "cantarell".to_string();
                    fonts.font_data.insert(
                        font_name.clone(),
                        std::sync::Arc::new(egui::FontData::from_owned(data)),
                    );
                    fonts
                        .families
                        .entry(egui::FontFamily::Proportional)
                        .or_default()
                        .insert(0, font_name);
                    fonts
                        .families
                        .entry(egui::FontFamily::Name("GTK".into()))
                        .or_default()
                        .insert(0, "cantarell".into());
                    has_cantarell = true;
                }
            }
        }

        let is_emoji = families.iter().any(|n| n.contains("emoji"));
        if !is_emoji { continue; }

        let is_color = families.iter().any(|n| n.contains("color"));
        if is_color { continue; }

        if let fontdb::Source::File(path) = &face.source {
            if let Ok(data) = std::fs::read(path) {
                let font_name = format!("emoji_{}", face.index);
                fonts.font_data.insert(
                    font_name.clone(),
                    std::sync::Arc::new(egui::FontData::from_owned(data)),
                );
                fonts
                    .families
                    .entry(egui::FontFamily::Proportional)
                    .or_default()
                    .push(font_name);
                loaded_emoji += 1;
            }
        }
    }

    if !has_cantarell {
        info!("Cantarell no encontrada, usando fuente sans-serif por defecto.");
    }

    if loaded_emoji == 0 {
        tracing::warn!("No se encontró una fuente emoji monocromática compatible. \
                        En Arch Linux: paru -S ttf-noto-emoji-monochrome");
    }

    egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
    info!("Phosphor font registrada. Familias Proportional: {:?}",
        fonts.families.get(&egui::FontFamily::Proportional));

    ctx.set_fonts(fonts);
}

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
        Box::new(|cc| {
            setup_fonts(&cc.egui_ctx);
            Ok(Box::new(TypshowApp::new(shared_state)))
        }),
    )?;

    info!("Aplicación terminada con éxito.");
    Ok(())
}