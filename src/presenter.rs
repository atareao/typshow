use eframe::App;
use egui::containers::scroll_area::State as ScrollState;
use egui::{
    CentralPanel, Context, TopBottomPanel, Ui, Button, Margin, Key, PointerButton,
    Sense, SidePanel, ScrollArea, Vec2, ViewportId, ViewportBuilder, ViewportClass,
    ViewportCommand, TextEdit, RichText, Frame,
};
use crate::app::SharedState;
use crate::theme::apply_gtk4_style;
use crate::fullscreen::FullscreenApp;
use tracing::{info, debug, error, warn};
use std::thread;
use egui_phosphor::regular as icons;

pub struct TypshowApp {
    state: SharedState,
    fullscreen_app: FullscreenApp,
    show_fullscreen: bool,
    projection_moved_to_secondary: bool,
    current_filename: Option<String>,
    notes_needs_scroll: bool,
}

impl TypshowApp {
    pub fn new(state: SharedState) -> Self {
        let fullscreen_app = FullscreenApp::new(state.clone());
        Self {
            state,
            fullscreen_app,
            show_fullscreen: false,
            projection_moved_to_secondary: false,
            current_filename: None,
            notes_needs_scroll: false,
        }
    }

    fn navigate(&mut self, action: impl Fn(&mut crate::app::AppState)) {
        let mut state = self.state.lock();

        if state.notes.editing && state.notes.dirty {
            let _ = state.notes.save_current();
        }

        action(&mut state);
        let page = state.current_page;
        state.notes.load_page(page);
        self.notes_needs_scroll = true;
    }

    fn draw_navigation_bar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            let mut state = self.state.lock();

            if ui.button(format!("{}  Abrir...", icons::FOLDER_OPEN)).clicked() {
                debug!("Hiciste clic en '📂 Abrir...'. Iniciando diálogo de selección de archivo...");
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Presentaciones", &["typ"])
                    .pick_file()
                {
                    info!("Archivo seleccionado: {:?}", path);
                    if let Some(path_str) = path.to_str() {
                        match state.load_file(path_str) {
                            Ok(()) => {
                                info!("Documento '{}' cargado con éxito.", path_str);
                                self.notes_needs_scroll = true;
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
                return;
            }

            ui.separator();

            let theme_icon = if state.dark_mode { icons::MOON } else { icons::SUN };
            if ui.button(theme_icon).clicked() {
                state.dark_mode = !state.dark_mode;
                apply_gtk4_style(ui.ctx(), state.dark_mode);
                ui.ctx().request_repaint();
            }

            ui.separator();

            let fs_label = if self.show_fullscreen {
                format!("{} Ocultar", icons::DESKTOP)
            } else {
                format!("{} Mostrar", icons::DESKTOP)
            };
            if ui.button(fs_label).clicked() {
                self.show_fullscreen = !self.show_fullscreen;
                ui.ctx().request_repaint();
            }

            ui.separator();

            if ui.add_enabled(state.current_page > 0, Button::new(format!("{} Inicio", icons::SKIP_BACK))).clicked() {
                drop(state);
                self.navigate(|s| s.first_page());
                ui.ctx().request_repaint();
                return;
            }

            if ui.add_enabled(state.current_page > 0, Button::new(format!("{} Anterior", icons::CARET_LEFT))).clicked() {
                drop(state);
                self.navigate(|s| s.prev_page());
                ui.ctx().request_repaint();
                return;
            }

            if state.total_pages > 0 {
                ui.label(format!("Página {} / {}", state.current_page + 1, state.total_pages));
            } else {
                ui.label("Sin documento");
            }

            if ui.add_enabled(state.current_page + 1 < state.total_pages, Button::new(format!("Siguiente {}", icons::CARET_RIGHT))).clicked() {
                drop(state);
                self.navigate(|s| s.next_page());
                ui.ctx().request_repaint();
                return;
            }

            if ui.add_enabled(state.current_page + 1 < state.total_pages, Button::new(format!("Final {}", icons::SKIP_FORWARD))).clicked() {
                drop(state);
                self.navigate(|s| s.last_page());
                ui.ctx().request_repaint();
            }
        });
    }

    fn handle_page_click(&mut self, ctx: &Context, response: &egui::Response) {
        if self.state.lock().notes.editing { return; }
        if response.clicked_by(PointerButton::Primary) {
            self.navigate(|s| s.prev_page());
            ctx.request_repaint();
        }
        if response.secondary_clicked() {
            self.navigate(|s| s.next_page());
            ctx.request_repaint();
        }
    }

    fn draw_page_view(&mut self, ui: &mut Ui, ctx: &Context, page_idx: usize, label: &str) {
        ui.vertical_centered(|ui| {
            ui.label(label);
            let texture = self.state.lock().renderer.get_page(ctx, page_idx);
            if let Some(texture) = texture {
                let remaining = ui.available_size();
                let response = ui.add(
                    egui::Image::new(&texture)
                        .fit_to_exact_size(remaining)
                        .sense(Sense::click()),
                );
                drop(texture);
                self.handle_page_click(ctx, &response);
            } else {
                ui.label("Cargando página...");
            }
        });
    }
}

impl App for TypshowApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        let dark_mode = self.state.lock().dark_mode;
        apply_gtk4_style(ctx, dark_mode);

        let filename = {
            let state = self.state.lock();
            state.file_path.as_ref().and_then(|p| {
                std::path::Path::new(p)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|s| s.to_string())
            })
        };

        if self.current_filename != filename {
            self.current_filename = filename.clone();
            let title = match &filename {
                Some(name) => format!("Typshow - Controlador ({})", name),
                None => "Typshow - Controlador".to_string(),
            };
            ctx.send_viewport_cmd_to(ViewportId::ROOT, ViewportCommand::Title(title));
        }

        let editing = self.state.lock().notes.editing;

        let mut next = false;
        let mut prev = false;
        let mut first = false;
        let mut last = false;
        let mut esc = false;

        if !editing {
            ctx.input(|i| {
                if i.key_pressed(Key::Escape) { esc = true; }
                if i.key_pressed(Key::H) { first = true; }
                if i.key_pressed(Key::L) { last = true; }
                if i.key_pressed(Key::J) || i.key_pressed(Key::ArrowRight) || i.key_pressed(Key::PageDown) { next = true; }
                if i.key_pressed(Key::K) || i.key_pressed(Key::ArrowLeft) || i.key_pressed(Key::PageUp) { prev = true; }
            });
        }

        if esc {
            self.show_fullscreen = false;
            ctx.send_viewport_cmd_to(
                ViewportId::from_hash_of("fullscreen_viewport"),
                egui::ViewportCommand::Close,
            );
        }

        if first { self.navigate(|s| s.first_page()); ctx.request_repaint(); }
        if last { self.navigate(|s| s.last_page()); ctx.request_repaint(); }
        if next { self.navigate(|s| s.next_page()); ctx.request_repaint(); }
        if prev { self.navigate(|s| s.prev_page()); ctx.request_repaint(); }

        TopBottomPanel::top("nav_bar").show(ctx, |ui| {
            ui.add_space(6.0);
            self.draw_navigation_bar(ui);
            ui.add_space(6.0);
        });

        let state = self.state.clone();

        SidePanel::right("notes_panel")
            .resizable(true)
            .default_width(300.0)
            .min_width(200.0)
            .show(ctx, |ui| {
                ui.add_space(4.0);
                ui.strong("Notas");
                ui.separator();
                ui.add_space(4.0);

                let has_doc = state.lock().file_path.is_some();
                if !has_doc {
                    ui.colored_label(egui::Color32::GRAY, "Cargue una presentación para ver las notas.");
                    return;
                }

                let notes_available = state.lock().notes.has_content();
                if !notes_available {
                    ui.colored_label(egui::Color32::GRAY, "No se pudo inicializar la base de datos de notas.");
                    return;
                }

                // Show/edit heading for current page
                {
                    let mut sg = state.lock();
                    if sg.notes.heading_editing {
                        ui.horizontal(|ui| {
                            ui.add_sized(
                                egui::vec2(150.0, 20.0),
                                TextEdit::singleline(&mut sg.notes.heading_edit_buffer)
                                    .hint_text("Título de la nota"),
                            );
                            if ui.button(icons::FLOPPY_DISK).clicked() {
                                let _ = sg.notes.save_heading();
                                ui.ctx().request_repaint();
                            }
                            if ui.button(icons::X_CIRCLE).clicked() {
                                sg.notes.cancel_edit_heading();
                                ui.ctx().request_repaint();
                            }
                        });
                    } else {
                        let heading = sg.notes.heading.clone();
                        let page_num = sg.current_page + 1;
                        drop(sg);
                        if let Some(h) = &heading {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(format!("Pág. {}:", page_num)).size(14.0).strong());
                                ui.label(RichText::new(h).size(14.0).strong());
                                if ui.button(icons::PENCIL_SIMPLE).clicked() {
                                    state.lock().notes.start_edit_heading();
                                    ui.ctx().request_repaint();
                                }
                            });
                            ui.separator();
                            ui.add_space(2.0);
                        }
                    }
                }

                // Editing controls
                let editing = state.lock().notes.editing;
                ui.horizontal(|ui| {
                    if editing {
                        let mut state_guard = state.lock();
                        if ui.button(format!("{}  Guardar", icons::FLOPPY_DISK)).clicked() {
                            let _ = state_guard.notes.save_current();
                            ui.ctx().request_repaint();
                        }
                        if ui.button(format!("{}  Cancelar", icons::X_CIRCLE)).clicked() {
                            state_guard.notes.cancel_edit();
                            ui.ctx().request_repaint();
                        }
                    } else {
                        if ui.button(format!("{}  Editar", icons::PENCIL_SIMPLE)).clicked() {
                            state.lock().notes.start_edit();
                            ui.ctx().request_repaint();
                        }
                        if ui.button(format!("{}  Nueva", icons::PLUS)).clicked() {
                            match state.lock().notes.insert_note() {
                                Ok(()) => ui.ctx().request_repaint(),
                                Err(e) => warn!("Error insertando nota: {}", e),
                            }
                        }
                        if ui.button(format!("{}  Eliminar", icons::TRASH)).clicked() {
                            match state.lock().notes.delete_current() {
                                Ok(()) => ui.ctx().request_repaint(),
                                Err(e) => warn!("Error eliminando nota: {}", e),
                            }
                        }
                        if ui.button(format!("{}  Reimportar", icons::ARROWS_CLOCKWISE)).clicked() {
                            match state.lock().reimport_notes() {
                                Ok(()) => ui.ctx().request_repaint(),
                                Err(e) => warn!("Error reimportando notas: {}", e),
                            }
                        }
                    }
                });
                ui.add_space(4.0);

                let scroll_id = ui.make_persistent_id(egui::Id::new("notes_scroll"));

                Frame::NONE
                    .inner_margin(Margin::same(4))
                    .show(ui, |ui| {
                        ScrollArea::vertical()
                            .id_salt("notes_scroll")
                            .show(ui, |ui| {
                                let mut state_guard = state.lock();
                                if state_guard.notes.editing {
                                    let text = &mut state_guard.notes.edit_buffer;
                                    let response = ui.add_sized(
                                        ui.available_size(),
                                        TextEdit::multiline(text)
                                            .desired_width(f32::INFINITY)
                                            .hint_text("Escriba sus notas aquí..."),
                                    );
                                    if response.changed() {
                                        state_guard.notes.dirty = true;
                                    }
                                } else {
                                    state_guard.notes.draw(ui);
                                }
                            });

                        if self.notes_needs_scroll {
                            let mut scroll_state = ScrollState::load(ctx, scroll_id).unwrap_or_default();
                            scroll_state.offset = Vec2::ZERO;
                            scroll_state.store(ctx, scroll_id);
                            self.notes_needs_scroll = false;
                        }
                    });
            });

        CentralPanel::default().show(ctx, |ui| {
            let has_doc = state.lock().file_path.is_some();
            if !has_doc {
                ui.centered_and_justified(|ui| {
                    ui.heading(format!("Haga clic en '{} Abrir...' para cargar una presentación.", icons::FOLDER_OPEN));
                });
                return;
            }

            ui.style_mut().spacing.item_spacing = egui::vec2(0.0, 0.0);
            ui.style_mut().spacing.window_margin = Margin::same(4);

            let total_height = ui.available_height();
            let half_height = total_height / 2.0;

            let (total_pages, current_page_idx) = {
                let state_guard = state.lock();
                (state_guard.total_pages, state_guard.current_page)
            };

            ui.allocate_ui(Vec2::new(ui.available_width(), half_height), |ui| {
                self.draw_page_view(ui, ctx, current_page_idx, "Página Actual:");
            });

            ui.allocate_ui(Vec2::new(ui.available_width(), ui.available_height()), |ui| {
                let next_page_idx = current_page_idx + 1;
                if next_page_idx < total_pages {
                    self.draw_page_view(ui, ctx, next_page_idx, "Siguiente Página:");
                } else {
                    ui.centered_and_justified(|ui| {
                        ui.colored_label(egui::Color32::GRAY, "Fin de la presentación");
                    });
                }
            });
        });

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
                },
            );

            if just_opened && !self.projection_moved_to_secondary {
                self.projection_moved_to_secondary = true;
                thread::spawn(|| {
                    thread::sleep(std::time::Duration::from_millis(150));
                    let _ = std::process::Command::new("niri")
                        .args(&["msg", "action", "move-column-to-monitor-next"])
                        .status();
                    thread::sleep(std::time::Duration::from_millis(50));
                    let _ = std::process::Command::new("niri")
                        .args(&["msg", "action", "focus-monitor-previous"])
                        .status();
                });
            }
        } else {
            self.projection_moved_to_secondary = false;
        }
    }
}