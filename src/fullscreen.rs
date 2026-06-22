use egui::{CentralPanel, Context, Frame, Key, ViewportCommand, ViewportId, Margin, PointerButton, Sense};
use crate::app::SharedState;
use tracing::info;

pub struct FullscreenApp {
    state: SharedState,
}

impl FullscreenApp {
    pub fn new(state: SharedState) -> Self {
        Self { state }
    }

    pub fn show(&mut self, ctx: &Context) {
        // 1. Capturar atajos de teclado de forma segura SIN BLOQUEOS concurrentes (Seteando banderas booleanas simples)
        let mut next = false;
        let mut prev = false;
        let mut first = false;
        let mut last = false;
        let mut esc = false;

        ctx.input(|i| {
            if i.key_pressed(Key::Escape) {
                esc = true;
            }
            if i.key_pressed(Key::H) {
                first = true;
            }
            if i.key_pressed(Key::L) {
                last = true;
            }
            if i.key_pressed(Key::J) || i.key_pressed(Key::ArrowRight) || i.key_pressed(Key::PageDown) {
                next = true;
            }
            if i.key_pressed(Key::K) || i.key_pressed(Key::ArrowLeft) || i.key_pressed(Key::PageUp) {
                prev = true;
            }
        });

        // 2. Procesar comandos de teclado de forma segura una vez liberado el candado del input de egui
        if esc {
            info!("Presionado 'Esc' en pantalla completa. Solicitando cierre del viewport de proyección.");
            ctx.send_viewport_cmd_to(
                ViewportId::from_hash_of("fullscreen_viewport"),
                ViewportCommand::Close
            );
        }

        if first || last || next || prev {
            let mut state = self.state.lock();
            if first {
                state.first_page();
            }
            if last {
                state.last_page();
            }
            if next {
                state.next_page();
            }
            if prev {
                state.prev_page();
            }
            ctx.request_repaint();
        }

        CentralPanel::default()
            .frame(Frame::NONE.fill(egui::Color32::BLACK))
            .show(ctx, |ui| {
                // Eliminar márgenes y padding internos del panel para lograr 100% de ocupación de pantalla
                ui.style_mut().spacing.item_spacing = egui::vec2(0.0, 0.0);
                ui.style_mut().spacing.window_margin = Margin::same(0);

                let has_doc = self.state.lock().file_path.is_some();
                if !has_doc {
                    ui.centered_and_justified(|ui| {
                        ui.colored_label(egui::Color32::GRAY, "Proyección de Presentación (Esperando documento...)");
                    });
                    return;
                }

                let current_page_idx = self.state.lock().current_page;

                // Obtenemos la textura de la página
                let texture = self.state.lock().renderer.get_page(
                    ctx,
                    current_page_idx,
                );

                if let Some(texture) = texture {
                    let available_size = ui.available_size();
                    
                    // Dibujamos la imagen del PDF centrada y justificada ocupando el 100% del espacio
                    ui.centered_and_justified(|ui| {
                        // Hacemos que la imagen misma sea interactiva usando el Sense de clic
                        let response = ui.add(
                            egui::Image::new(&texture)
                                .fit_to_exact_size(available_size)
                                .sense(Sense::click())
                        );

                        // 3. Capturar clics de ratón de forma local en la imagen sin consumir espacio antes
                        if response.clicked_by(PointerButton::Primary) {
                            info!("Proyección - Clic izquierdo en la diapositiva. Página anterior.");
                            self.state.lock().prev_page();
                        }
                        if response.secondary_clicked() {
                            info!("Proyección - Clic derecho en la diapositiva. Página siguiente.");
                            self.state.lock().next_page();
                        }
                    });
                }
            });
    }
}
