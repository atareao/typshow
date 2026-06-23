use eframe::App;
use egui::containers::scroll_area::State as ScrollState;
use egui::{
    CentralPanel, Context, TopBottomPanel, Ui, Button, Margin, Key, PointerButton,
    Sense, SidePanel, ScrollArea, Vec2, ViewportId, ViewportBuilder, ViewportClass,
    ViewportCommand,
};
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
    current_filename: Option<String>,
    notes_scroll_ratio: f32,
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
            notes_scroll_ratio: 0.0,
            notes_needs_scroll: false,
        }
    }

    fn navigate(&mut self, action: impl Fn(&mut crate::app::AppState)) {
        let mut state = self.state.lock();
        action(&mut state);
        let page = state.current_page;
        let total = state.total_pages;
        state.notes.update_scroll_target(page, total);
        self.notes_scroll_ratio = state.notes.scroll_ratio;
        self.notes_needs_scroll = true;
    }

    fn draw_navigation_bar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            let mut state = self.state.lock();

            if ui.button("📂 Abrir...").clicked() {
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
                                let total = state.total_pages;
                                state.notes.update_scroll_target(0, total);
                                self.notes_scroll_ratio = 0.0;
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

            let theme_icon = if state.dark_mode { "🌙" } else { "☀️" };
            if ui.button(theme_icon).clicked() {
                state.dark_mode = !state.dark_mode;
                apply_gtk4_style(ui.ctx(), state.dark_mode);
                ui.ctx().request_repaint();
            }

            ui.separator();

            let fs_label = if self.show_fullscreen { "🖥 Ocultar Proyección" } else { "🖥 Mostrar Proyección" };
            if ui.button(fs_label).clicked() {
                self.show_fullscreen = !self.show_fullscreen;
                ui.ctx().request_repaint();
            }

            ui.separator();

            if ui.add_enabled(state.current_page > 0, Button::new("⏮ Inicio")).clicked() {
                drop(state);
                self.navigate(|s| s.first_page());
                ui.ctx().request_repaint();
                return;
            }

            if ui.add_enabled(state.current_page > 0, Button::new("◀ Anterior")).clicked() {
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

            if ui.add_enabled(state.current_page + 1 < state.total_pages, Button::new("Siguiente ▶")).clicked() {
                drop(state);
                self.navigate(|s| s.next_page());
                ui.ctx().request_repaint();
                return;
            }

            if ui.add_enabled(state.current_page + 1 < state.total_pages, Button::new("Final ⏭")).clicked() {
                drop(state);
                self.navigate(|s| s.last_page());
                ui.ctx().request_repaint();
            }
        });
    }

    fn handle_page_click(&mut self, ctx: &Context, response: &egui::Response) {
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

        let mut next = false;
        let mut prev = false;
        let mut first = false;
        let mut last = false;
        let mut esc = false;

        ctx.input(|i| {
            if i.key_pressed(Key::Escape) { esc = true; }
            if i.key_pressed(Key::H) { first = true; }
            if i.key_pressed(Key::L) { last = true; }
            if i.key_pressed(Key::J) || i.key_pressed(Key::ArrowRight) || i.key_pressed(Key::PageDown) { next = true; }
            if i.key_pressed(Key::K) || i.key_pressed(Key::ArrowLeft) || i.key_pressed(Key::PageUp) { prev = true; }
        });

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
            ui.add_space(4.0);
            self.draw_navigation_bar(ui);
            ui.add_space(4.0);
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

                let notes_has_content = state.lock().notes.content.is_some();
                if !notes_has_content {
                    let path_display = state.lock().notes.path.clone()
                        .unwrap_or_else(|| "desconocido".to_string());
                    ui.colored_label(egui::Color32::GRAY, format!("No se encontraron notas en:\n{}", path_display));
                    return;
                }

                let scroll_id = ui.make_persistent_id(egui::Id::new("notes_scroll"));

                let viewport_height = ui.available_height();

                let output = ScrollArea::vertical()
                    .id_salt("notes_scroll")
                    .show(ui, |ui| {
                        let notes = &state.lock().notes;
                        notes.draw(ui);
                    });

                if self.notes_needs_scroll {
                    let content_height = output.content_size.y;
                    let max_scroll = (content_height - viewport_height).max(0.0);
                    let offset = (self.notes_scroll_ratio * content_height).min(max_scroll);
                    let mut scroll_state = ScrollState::load(ctx, scroll_id).unwrap_or_default();
                    scroll_state.offset = Vec2::new(0.0, offset);
                    scroll_state.store(ctx, scroll_id);
                    info!("NOTES SCROLL: offset={:.1} ratio={:.3} content_h={:.1} vp_h={:.1}", offset, self.notes_scroll_ratio, content_height, viewport_height);
                    self.notes_needs_scroll = false;
                }
            });

        CentralPanel::default().show(ctx, |ui| {
            let has_doc = state.lock().file_path.is_some();
            if !has_doc {
                ui.centered_and_justified(|ui| {
                    ui.heading("Haga clic en '📂 Abrir...' para cargar una presentación.");
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