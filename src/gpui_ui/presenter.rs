//! Ventana del presentador: página actual + siguiente, navegación, tema y notas.

use gpui::{
    AppContext, Bounds, Context, Entity, FocusHandle, KeyDownEvent, MouseButton, ObjectFit, Render,
    Subscription, Window, WindowBounds, WindowHandle, WindowOptions, div, img, prelude::*, px,
};

use super::audience::AudienceView;
use super::deck::Deck;
use super::keys::{apply_edit, apply_nav};
use super::theme::{Palette, palette};

/// Ejecuta `action` sobre el `Deck` compartido y notifica a los observers.
fn run<C: AppContext>(deck: &Entity<Deck>, action: fn(&mut Deck), cx: &mut C) {
    deck.update(cx, |deck, cx| {
        action(deck);
        cx.notify();
    });
}

/// Botón de acción principal (relleno de acento).
fn action_button(
    deck: &Entity<Deck>,
    label: &str,
    p: &Palette,
    action: fn(&mut Deck),
) -> impl IntoElement {
    let deck = deck.clone();
    div()
        .px_3()
        .py_1()
        .rounded_md()
        .bg(p.accent)
        .text_color(p.on_accent)
        .child(label.to_string())
        .on_mouse_down(MouseButton::Left, move |_event, _window, cx| {
            run(&deck, action, cx);
        })
}

/// Botón de navegación con estado habilitado/deshabilitado.
fn nav_button(
    deck: &Entity<Deck>,
    label: &str,
    enabled: bool,
    p: &Palette,
    action: fn(&mut Deck),
) -> gpui::AnyElement {
    let base = div().px_3().py_1().rounded_md().child(label.to_string());
    if enabled {
        let deck = deck.clone();
        base.bg(p.accent)
            .text_color(p.on_accent)
            .on_mouse_down(MouseButton::Left, move |_event, _window, cx| {
                run(&deck, action, cx);
            })
            .into_any_element()
    } else {
        base.bg(p.surface).text_color(p.muted).into_any_element()
    }
}

/// Vista de la ventana del presentador.
pub struct PresenterView {
    deck: Entity<Deck>,
    focus_handle: FocusHandle,
    audience: Option<WindowHandle<AudienceView>>,
    _sub: Subscription,
    _close_sub: Subscription,
}

impl PresenterView {
    /// Crea la vista del presentador para la `deck` compartida.
    pub fn new(deck: Entity<Deck>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        focus_handle.focus(window, cx);
        let _sub = cx.observe(&deck, |_this, _deck, cx| cx.notify());

        // Mantiene `audience`/`projection` sincronizados cuando la ventana de
        // audiencia se cierra (Esc, botón del compositor, etc.).
        let this = cx.entity().downgrade();
        let _close_sub = cx.on_window_closed(move |cx, window_id| {
            this.update(cx, |view, cx| {
                if view.audience.map(|h| h.window_id()) == Some(window_id) {
                    view.audience = None;
                    view.deck.update(cx, |deck, cx| {
                        deck.projection = false;
                        cx.notify();
                    });
                    cx.notify();
                }
            })
            .ok();
        });

        let mut view = Self {
            deck,
            focus_handle,
            audience: None,
            _sub,
            _close_sub,
        };
        if view.deck.read(cx).projection {
            view.open_audience(cx);
        }
        view
    }

    /// Abre la ventana de audiencia (fullscreen) y guarda su handle.
    fn open_audience(&mut self, cx: &mut Context<Self>) {
        let deck = self.deck.clone();
        self.audience = cx
            .open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Fullscreen(Bounds::default())),
                    ..Default::default()
                },
                move |window, cx| cx.new(|cx| AudienceView::new(deck, window, cx)),
            )
            .ok();
    }

    /// Cierra la ventana de audiencia si está abierta.
    fn close_audience(&mut self, cx: &mut Context<Self>) {
        if let Some(handle) = self.audience.take() {
            let _ = handle.update(cx, |_view, window, _cx| window.remove_window());
        }
    }

    /// Muestra u oculta la proyección, reconciliando el handle con el estado real.
    fn toggle_projection(&mut self, cx: &mut Context<Self>) {
        // `is_active` refleja el FOCO, no si la ventana sigue viva: comprobamos
        // pertenencia a `cx.windows()` para no dejar ventanas huérfanas ni duplicar.
        if let Some(handle) = self.audience {
            let id = handle.window_id();
            if cx.windows().iter().all(|w| w.window_id() != id) {
                self.audience = None;
            }
        }
        if self.audience.is_some() {
            self.close_audience(cx);
            self.deck.update(cx, |deck, cx| {
                deck.projection = false;
                cx.notify();
            });
        } else {
            self.open_audience(cx);
            self.deck.update(cx, |deck, cx| {
                deck.projection = true;
                cx.notify();
            });
        }
    }

    /// Abre el diálogo de fichero y carga el `.typ` elegido.
    fn open_file_dialog(&mut self, cx: &mut Context<Self>) {
        let picked = rfd::FileDialog::new()
            .add_filter("Typst", &["typ"])
            .set_title("Abrir presentación Typst")
            .pick_file();
        let Some(path) = picked else {
            return;
        };
        let path = path.to_string_lossy().to_string();
        self.deck.update(cx, |deck, cx| {
            if let Err(msg) = deck.open_file(&path) {
                deck.error = Some(msg);
            }
            cx.notify();
        });
    }

    /// Ejecuta `f` sobre el `Deck` desde un handler de la vista.
    fn with_deck(&mut self, cx: &mut Context<Self>, f: impl FnOnce(&mut Deck)) {
        self.deck.update(cx, |deck, cx| {
            f(deck);
            cx.notify();
        });
    }

    /// Panel lateral de notas (ver/editar/acciones).
    fn notes_panel(&self, cx: &mut Context<Self>, p: &Palette) -> impl IntoElement {
        let notes = &self.deck.read(cx).notes;
        let available = notes.has_content();
        let editing = notes.editing;
        let heading = notes.heading.clone();
        let content = notes.note_content.clone().unwrap_or_default();
        let edit_buffer = notes.edit_buffer.clone();

        let body: gpui::AnyElement = if !available {
            div()
                .text_color(p.muted)
                .child("No hay base de datos de notas para este documento")
                .into_any_element()
        } else if editing {
            div()
                .flex()
                .flex_col()
                .gap_2()
                .child(
                    div()
                        .flex_1()
                        .min_h_0()
                        .p_2()
                        .bg(p.surface)
                        .border_1()
                        .border_color(p.border)
                        .rounded_md()
                        .id("notes-edit-body")
                        .overflow_y_scroll()
                        .text_color(p.fg)
                        .child(if edit_buffer.is_empty() {
                            "Escribe la nota…".to_string()
                        } else {
                            edit_buffer
                        }),
                )
                .child(
                    div()
                        .flex()
                        .gap_2()
                        .child(
                            div()
                                .px_3()
                                .py_1()
                                .rounded_md()
                                .bg(p.accent)
                                .text_color(p.on_accent)
                                .child("Guardar")
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(|this, _e, _w, cx| {
                                        this.with_deck(cx, |d| {
                                            if let Err(e) = d.notes.save_current() {
                                                d.error = Some(e.to_string());
                                            }
                                        })
                                    }),
                                ),
                        )
                        .child(
                            div()
                                .px_3()
                                .py_1()
                                .rounded_md()
                                .bg(p.surface)
                                .text_color(p.fg)
                                .child("Cancelar")
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(|this, _e, _w, cx| {
                                        this.with_deck(cx, |d| d.notes.cancel_edit())
                                    }),
                                ),
                        ),
                )
                .into_any_element()
        } else {
            let content = if content.trim().is_empty() {
                "Las notas están vacías".to_string()
            } else {
                content
            };
            div()
                .flex_1()
                .min_h_0()
                .p_2()
                .bg(p.surface)
                .border_1()
                .border_color(p.border)
                .rounded_md()
                .id("notes-view-body")
                .overflow_y_scroll()
                .text_color(if content == "Las notas están vacías" {
                    p.muted
                } else {
                    p.fg
                })
                .child(content)
                .into_any_element()
        };

        let actions = div()
            .flex()
            .gap_2()
            .flex_wrap()
            .child(
                div()
                    .px_3()
                    .py_1()
                    .rounded_md()
                    .bg(p.accent)
                    .text_color(p.on_accent)
                    .child("Editar")
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _e, _w, cx| {
                            this.with_deck(cx, |d| d.notes.start_edit())
                        }),
                    ),
            )
            .child(
                div()
                    .px_3()
                    .py_1()
                    .rounded_md()
                    .bg(p.surface)
                    .text_color(p.fg)
                    .child("Nueva")
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _e, _w, cx| {
                            this.with_deck(cx, |d| {
                                if let Err(e) = d.notes.insert_note() {
                                    d.error = Some(e.to_string());
                                }
                            })
                        }),
                    ),
            )
            .child(
                div()
                    .px_3()
                    .py_1()
                    .rounded_md()
                    .bg(p.surface)
                    .text_color(p.fg)
                    .child("Eliminar")
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _e, _w, cx| {
                            this.with_deck(cx, |d| {
                                if let Err(e) = d.notes.delete_current() {
                                    d.error = Some(e.to_string());
                                }
                            })
                        }),
                    ),
            )
            .child(
                div()
                    .px_3()
                    .py_1()
                    .rounded_md()
                    .bg(p.surface)
                    .text_color(p.fg)
                    .child("Reimportar")
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _e, _w, cx| {
                            this.with_deck(cx, |d| {
                                if let Err(e) = d.reimport_notes() {
                                    d.error = Some(e);
                                }
                            })
                        }),
                    ),
            );

        div()
            .w(px(320.))
            .h_full()
            .flex()
            .flex_col()
            .gap_2()
            .pl_3()
            .border_l_1()
            .border_color(p.border)
            .child(
                div()
                    .flex()
                    .flex_row()
                    .justify_between()
                    .items_center()
                    .child(div().child("Notas")),
            )
            .child(if let Some(h) = heading {
                div().text_sm().text_color(p.muted).child(h)
            } else {
                div().text_sm().text_color(p.muted).child("(sin título)")
            })
            .child(body)
            .children((available && !editing).then_some(actions))
    }
}

impl Render for PresenterView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (page, total, dark, error, notes_visible, projection) = {
            let deck = self.deck.read(cx);
            (
                deck.page,
                deck.total,
                deck.dark,
                deck.error.clone(),
                deck.notes_visible,
                deck.projection,
            )
        };
        let (image, next_image) = self
            .deck
            .update(cx, |deck, _cx| (deck.current_image(), deck.next_image()));
        let (has_prev, has_next) = {
            let deck = self.deck.read(cx);
            (deck.has_prev(), deck.has_next())
        };
        let editing = self.deck.read(cx).notes.editing;

        let p = palette(dark);
        let total_label = if total == 0 { 1 } else { total };
        let deck = self.deck.clone();

        let current_pane = div()
            .flex_1()
            .min_h_0()
            .flex()
            .flex_col()
            .child(div().text_sm().text_color(p.muted).child("Actual"))
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .justify_center()
                    .items_center()
                    .p_8()
                    .child(match image {
                        Some(image) => img(image)
                            .size_full()
                            .object_fit(ObjectFit::Contain)
                            .into_any_element(),
                        None => div()
                            .text_color(p.fg)
                            .child(error.clone().unwrap_or_else(|| "Sin documento".to_string()))
                            .into_any_element(),
                    }),
            );

        let next_pane = div()
            .flex_1()
            .min_h_0()
            .flex()
            .flex_col()
            .border_t_1()
            .border_color(p.border)
            .child(div().text_sm().text_color(p.muted).child("Siguiente"))
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .justify_center()
                    .items_center()
                    .p_8()
                    .child(match next_image {
                        Some(image) => img(image)
                            .size_full()
                            .object_fit(ObjectFit::Contain)
                            .into_any_element(),
                        None => div()
                            .text_sm()
                            .text_color(p.muted)
                            .child(if total == 0 {
                                String::new()
                            } else {
                                "Fin de la presentacion".to_string()
                            })
                            .into_any_element(),
                    }),
            );

        let page_area = div()
            .flex_1()
            .min_h_0()
            .flex()
            .flex_col()
            .on_mouse_down(MouseButton::Left, {
                let deck = deck.clone();
                move |_event, _window, cx| {
                    if !deck.read(cx).notes.editing {
                        run(&deck, Deck::prev, cx);
                    }
                }
            })
            .on_mouse_down(MouseButton::Right, {
                let deck = deck.clone();
                move |_event, _window, cx| {
                    if !deck.read(cx).notes.editing {
                        run(&deck, Deck::next, cx);
                    }
                }
            })
            .child(current_pane)
            .child(next_pane);

        let main_row = div()
            .flex()
            .flex_row()
            .flex_1()
            .min_h_0()
            .child(page_area)
            .children(notes_visible.then(|| self.notes_panel(cx, &p)));

        let projection_label = if projection {
            "Ocultar proyección"
        } else {
            "Mostrar proyección"
        };

        div()
            .flex()
            .flex_col()
            .gap_3()
            .p_4()
            .size_full()
            .bg(p.bg)
            .text_color(p.fg)
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, _window, cx| {
                let editing = this.deck.read(cx).notes.editing;
                if editing {
                    apply_edit(&this.deck, event, cx);
                } else {
                    apply_nav(&this.deck, event, cx);
                }
            }))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .justify_between()
                    .items_center()
                    .child(format!("Presenter - pagina {}/{total_label}", page + 1))
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap_2()
                            .child(
                                div()
                                    .px_3()
                                    .py_1()
                                    .rounded_md()
                                    .bg(p.surface)
                                    .text_color(p.fg)
                                    .child("Abrir")
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _e, _w, cx| this.open_file_dialog(cx)),
                                    ),
                            )
                            .child(
                                div()
                                    .px_3()
                                    .py_1()
                                    .rounded_md()
                                    .bg(p.surface)
                                    .text_color(p.fg)
                                    .child(projection_label)
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _e, _w, cx| this.toggle_projection(cx)),
                                    ),
                            )
                            .child(
                                div()
                                    .px_3()
                                    .py_1()
                                    .rounded_md()
                                    .bg(if notes_visible { p.accent } else { p.surface })
                                    .text_color(if notes_visible { p.on_accent } else { p.fg })
                                    .child("Notas")
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(|this, _e, _w, cx| {
                                            this.with_deck(cx, Deck::toggle_notes)
                                        }),
                                    ),
                            )
                            .child(action_button(&deck, "Tema", &p, Deck::toggle_theme)),
                    ),
            )
            .children(error.clone().map(|e| {
                div()
                    .px_3()
                    .py_1()
                    .rounded_md()
                    .bg(p.danger)
                    .text_color(p.on_accent)
                    .child(e)
            }))
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(nav_button(&deck, "Inicio", has_prev, &p, Deck::first))
                    .child(nav_button(&deck, "Anterior", has_prev, &p, Deck::prev))
                    .child(nav_button(&deck, "Siguiente", has_next, &p, Deck::next))
                    .child(nav_button(&deck, "Final", has_next, &p, Deck::last)),
            )
            .children(editing.then(|| {
                div()
                    .text_sm()
                    .text_color(p.accent)
                    .child("Editando nota — Esc para cancelar")
            }))
            .child(main_row)
    }
}
