use eframe::App;
use egui::{CentralPanel, Context, TopBottomPanel, Ui, Button, Margin, Key, ViewportId, ViewportBuilder, ViewportClass};
use crate::app::SharedState;
use crate::theme::apply_gtk4_style;
use crate::fullscreen::FullscreenApp;
use tracing::{info, debug, error};
use std::thread;

pub struct TypshowApp {
    state: SharedState,
    fullscreen_app: FullscreenApp,
    show_fullscreen: bool,
    projection_moved_to_secondary: bool,
}

impl TypshowApp {
    pub fn new(state: SharedState) -> Self {
        let fullscreen_app = FullscreenApp::new(state.clone());
        Self {
            state,
            fullscreen_app,
            show_fullscreen: true,
            projection_moved_to_secondary: false,
        }
    }

    fn draw_navigation_bar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            let mut state = self.state.lock();

            // Botón para cargar nuevo PDF
            if ui.button("📂 Abrir...").clicked() {
                debug!("Hiciste clic en '📂 Abrir...'. Iniciando diálogo de selección de archivo...");
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Presentaciones", &["pdf", "typ"])
                    .pick_file()
                {
                    info!("Archivo seleccionado: {:?}", path);
                    if let Some(path_str) = path.to_str() {
                        match state.load_file(path_str) {
                            Ok(()) => {
                                info!("Documento '{}' cargado con éxito.", path_str);
                                ui.ctx().request_repaint();
                            }
                            Err(e) => {
                                error!("Error al cargar el documento '{}': {:?}", path_str, e);
                            }
                        }
                    }
                } else {
                    debug!("Diálogo de selección de archivo cancelado por el usuario.");
                }
            }

            ui.separator();

            // Toggle para modo oscuro
            let theme_icon = if state.dark_mode { "🌙" } else { "☀️" };
            if ui.button(theme_icon).clicked() {
                state.dark_mode = !state.dark_mode;
                info!("Modo oscuro configurado en: {}", state.dark_mode);
                apply_gtk4_style(ui.ctx(), state.dark_mode);
                ui.ctx().request_repaint(); // Redibujar de inmediato
            }

            ui.separator();

            // Toggle para pantalla completa de la audiencia
            let fs_label = if self.show_fullscreen { "🖥 Ocultar Proyección" } else { "🖥 Mostrar Proyección" };
            if ui.button(fs_label).clicked() {
                self.show_fullscreen = !self.show_fullscreen;
                info!("Visualización de proyección secundaria configurada en: {}", self.show_fullscreen);
                ui.ctx().request_repaint(); // Redibujar
            }

            ui.separator();

            // 4 botones de navegación
            if ui.add_enabled(state.current_page > 0, Button::new("⏮ Inicio")).clicked() {
                state.first_page();
                ui.ctx().request_repaint();
            }

            if ui.add_enabled(state.current_page > 0, Button::new("◀ Anterior")).clicked() {
                state.prev_page();
                ui.ctx().request_repaint();
            }

            if state.total_pages > 0 {
                ui.label(format!("Página {} / {}", state.current_page + 1, state.total_pages));
            } else {
                ui.label("Sin documento");
            }

            if ui.add_enabled(state.current_page + 1 < state.total_pages, Button::new("Siguiente ▶")).clicked() {
                state.next_page();
                ui.ctx().request_repaint();
            }

            if ui.add_enabled(state.current_page + 1 < state.total_pages, Button::new("Final ⏭")).clicked() {
                state.last_page();
                ui.ctx().request_repaint();
            }
        });
    }
}

impl App for TypshowApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        let dark_mode = self.state.lock().dark_mode;
        apply_gtk4_style(ctx, dark_mode);

        // 1. Capturar entradas de teclado de forma segura SIN BLOQUEOS concurrentes (Seteando banderas booleanas simples)
        let mut next = false;
        let mut prev = false;

        ctx.input(|i| {
            if i.key_pressed(Key::ArrowRight) || i.key_pressed(Key::PageDown) {
                next = true;
            }
            if i.key_pressed(Key::ArrowLeft) || i.key_pressed(Key::PageUp) {
                prev = true;
            }
        });

        // 2. Procesar comandos de teclado de forma segura una vez liberado el candado del input de egui
        if next || prev {
            let mut state = self.state.lock();
            if next {
                state.next_page();
            }
            if prev {
                state.prev_page();
            }
            ctx.request_repaint();
        }

        TopBottomPanel::top("nav_bar").show(ctx, |ui| {
            ui.add_space(4.0);
            self.draw_navigation_bar(ui);
            ui.add_space(4.0);
        });

        CentralPanel::default().show(ctx, |ui| {
            let has_doc = self.state.lock().document.is_some();
            if !has_doc {
                ui.centered_and_justified(|ui| {
                    ui.heading("Haga clic en '📂 Abrir...' para cargar una presentación.");
                });
                return;
            }

            // Configurar márgenes y padding internos del panel para optimizar área al máximo
            ui.style_mut().spacing.item_spacing = egui::vec2(0.0, 0.0);
            ui.style_mut().spacing.window_margin = Margin::same(4);

            let total_height = ui.available_height();
            let half_height = total_height / 2.0;

            // Extraemos los campos de AppState que necesitamos para evitar problemas de préstamos múltiples (borrow checker)
            let (file_path, total_pages, current_page_idx) = {
                let state_guard = self.state.lock();
                (state_guard.file_path.clone(), state_guard.total_pages, state_guard.current_page)
            };

            // 1. Panel Superior (Página Actual) - Delimitado exactamente en altura (50%)
            ui.allocate_ui(
                egui::vec2(ui.available_width(), half_height),
                |ui| {
                    ui.vertical_centered(|ui| {
                        ui.label("Página Actual:");
                        
                        // Obtenemos la textura de la página de forma segura utilizando los campos copiados
                        let texture = self.state.lock().renderer.get_page(
                            ctx,
                            &file_path,
                            total_pages,
                            current_page_idx,
                        );

                        if let Some(texture) = texture {
                            let remaining_size = ui.available_size();
                            ui.add(egui::Image::new(&texture).fit_to_exact_size(remaining_size));
                        } else {
                            ui.label("Cargando página...");
                        }
                    });
                }
            );

            // 2. Panel Inferior (Siguiente Página Preview) - Delimitado exactamente en altura (50%)
            ui.allocate_ui(
                egui::vec2(ui.available_width(), ui.available_height()),
                |ui| {
                    ui.vertical_centered(|ui| {
                        ui.label("Siguiente Página:");
                        
                        let next_page_idx = current_page_idx + 1;
                        if next_page_idx < total_pages {
                            // Obtenemos la textura de la vista previa de forma segura utilizando los campos copiados
                            let texture = self.state.lock().renderer.get_page(
                                ctx,
                                &file_path,
                                total_pages,
                                next_page_idx,
                            );

                            if let Some(texture) = texture {
                                let remaining_size = ui.available_size();
                                ui.add(egui::Image::new(&texture).fit_to_exact_size(remaining_size));
                            } else {
                                ui.label("Cargando página...");
                            }
                        } else {
                            ui.centered_and_justified(|ui| {
                                ui.colored_label(egui::Color32::GRAY, "Fin de la presentación");
                            });
                        }
                    });
                }
            );
        });

        // Mostrar viewport de pantalla completa si está activo
        if self.show_fullscreen {
            let builder = ViewportBuilder::default()
                .with_title("Audience View - Fullscreen")
                .with_decorations(false)
                .with_fullscreen(true);

            let mut just_opened = false;

            ctx.show_viewport_immediate(
                ViewportId::from_hash_of("fullscreen_viewport"),
                builder,
                |ctx, class| {
                    if class == ViewportClass::Immediate {
                        self.fullscreen_app.show(ctx);
                        just_opened = true;
                    }
                }
            );

            // Si se acaba de abrir el viewport de proyección y estamos bajo Niri/Wayland,
            // le enviamos un mensaje de control nativo IPC a Niri para mover la columna de proyección
            // a la otra pantalla física secundaria disponible.
            if just_opened && !self.projection_moved_to_secondary {
                self.projection_moved_to_secondary = true;
                info!("Typshow - Ventana de proyección abierta. Moviendo de forma nativa en Niri al monitor secundario...");
                thread::spawn(|| {
                    // Esperar un instante corto para asegurar que Niri ha mapeado la ventana y le ha dado el foco inicial
                    thread::sleep(std::time::Duration::from_millis(150));
                    let _ = std::process::Command::new("niri")
                        .args(&["msg", "action", "move-column-to-monitor-next"])
                        .status();
                });
            }
        } else {
            // Si desactivamos la proyección, reiniciamos el flag para que se mueva de nuevo al volver a activarla
            self.projection_moved_to_secondary = false;
        }
    }
}
