//! Spike de viabilidad de GPUI para Typshow.
//!
//! Abre dos ventanas nativas (presenter + audiencia) que comparten el estado
//! de la presentacion (`Deck`) mediante una `Entity` de GPUI y renderizan la
//! pagina actual de un documento Typst como imagen.

use std::collections::HashMap;
use std::sync::Arc;

use gpui::{
    App, AppContext as _, Bounds, Context, Entity, FocusHandle, KeyBinding, MouseButton, ObjectFit,
    Render, RenderImage, Subscription, Window, WindowBounds, WindowOptions, actions, div, img,
    prelude::*, px, rgb, size,
};
use gpui_platform::application;
use typst_layout::PagedDocument;

actions!(
    gpui_spike,
    [FirstPage, PrevPage, NextPage, LastPage, ToggleTheme]
);

/// Estado compartido entre ambas ventanas: documento, pagina actual y cache.
struct Deck {
    page: usize,
    total: usize,
    dark: bool,
    doc: Option<PagedDocument>,
    cache: HashMap<usize, Arc<RenderImage>>,
    error: Option<String>,
}

impl Deck {
    fn empty() -> Self {
        Self {
            page: 0,
            total: 0,
            dark: false,
            doc: None,
            cache: HashMap::new(),
            error: None,
        }
    }

    fn with_error(msg: impl Into<String>) -> Self {
        let mut deck = Self::empty();
        deck.error = Some(msg.into());
        deck
    }

    fn load(path: &str) -> Self {
        let mut deck = Self::empty();
        match std::fs::read_to_string(path) {
            Ok(text) => {
                let world = typshow::TypstWorld::new(&text, path);
                match typst::compile::<PagedDocument>(&world).output {
                    Ok(doc) => {
                        deck.total = doc.pages().len();
                        deck.doc = Some(doc);
                    }
                    Err(diags) => {
                        let msg: Vec<String> = diags.iter().map(|d| format!("{d:?}")).collect();
                        deck.error = Some(format!("Error compilando Typst:\n{}", msg.join("\n")));
                    }
                }
            }
            Err(e) => deck.error = Some(format!("Error leyendo fichero: {e}")),
        }
        deck
    }

    fn first(&mut self) {
        self.page = 0;
    }

    fn prev(&mut self) {
        if self.page > 0 {
            self.page -= 1;
        }
    }

    fn next(&mut self) {
        if self.page + 1 < self.total {
            self.page += 1;
        }
    }

    fn last(&mut self) {
        if self.total > 0 {
            self.page = self.total - 1;
        }
    }

    fn toggle_theme(&mut self) {
        self.dark = !self.dark;
    }

    fn render_page(&mut self, idx: usize) -> Option<Arc<RenderImage>> {
        if let Some(image) = self.cache.get(&idx) {
            return Some(image.clone());
        }
        let page = self.doc.as_ref()?.pages().get(idx)?;
        let pt_width = page.frame.size().x.to_pt();
        let pixel_per_pt = if pt_width > 0.0 {
            1920.0 / pt_width
        } else {
            2.0
        };
        let options = typst_render::RenderOptions {
            pixel_per_pt: typst::utils::Scalar::new(pixel_per_pt),
            render_bleed: false,
        };
        let pixmap = typst_render::render(page, &options);
        let (width, height) = (pixmap.width(), pixmap.height());
        let mut data = pixmap.data().to_vec();
        rgba_to_bgra(&mut data);
        let rgba = image::RgbaImage::from_raw(width, height, data)?;
        let render_image = Arc::new(RenderImage::new(vec![image::Frame::new(rgba)]));
        self.cache.insert(idx, render_image.clone());
        Some(render_image)
    }

    fn current_image(&mut self) -> Option<Arc<RenderImage>> {
        self.render_page(self.page)
    }

    fn next_image(&mut self) -> Option<Arc<RenderImage>> {
        let next = self.page + 1;
        if next < self.total {
            self.render_page(next)
        } else {
            None
        }
    }
}

/// Ventana del presentador: pagina actual + controles de navegacion y tema.
struct PresenterView {
    deck: Entity<Deck>,
    focus_handle: FocusHandle,
    _sub: Subscription,
}

impl PresenterView {
    fn new(deck: Entity<Deck>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        focus_handle.focus(window, cx);
        let _sub = cx.observe(&deck, |_this, _deck, cx| cx.notify());
        Self {
            deck,
            focus_handle,
            _sub,
        }
    }

    fn apply(&mut self, cx: &mut Context<Self>, action: impl Fn(&mut Deck)) {
        self.deck.update(cx, |deck, cx| {
            action(deck);
            cx.notify();
        });
    }

    fn nav_button(
        &self,
        label: &'static str,
        cx: &mut Context<Self>,
        action: fn(&mut Deck),
    ) -> impl IntoElement {
        let deck = self.deck.clone();
        div()
            .px_4()
            .py_2()
            .bg(rgb(0x3584e4))
            .rounded_md()
            .text_color(rgb(0xffffff))
            .child(label)
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |_this, _event, _window, cx| {
                    deck.update(cx, |deck, cx| {
                        action(deck);
                        cx.notify();
                    });
                }),
            )
    }
}

impl Render for PresenterView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (page, total, dark, error) = {
            let deck = self.deck.read(cx);
            (deck.page, deck.total, deck.dark, deck.error.clone())
        };
        let (image, next_image) = self
            .deck
            .update(cx, |deck, _cx| (deck.current_image(), deck.next_image()));

        let bg = if dark { rgb(0x1e1e1e) } else { rgb(0xfafafa) };
        let fg = if dark { rgb(0xffffff) } else { rgb(0x1e1e1e) };
        let border = if dark { rgb(0x3a3a3a) } else { rgb(0xd0d0d0) };
        let total_label = if total == 0 { 1 } else { total };

        let current_pane = div()
            .flex_1()
            .min_h_0()
            .flex()
            .flex_col()
            .child(div().text_sm().child("Actual"))
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .justify_center()
                    .items_center()
                    .p_8()
                    .child(match image {
                        Some(image) => img(image).object_fit(ObjectFit::Contain).into_any_element(),
                        None => div()
                            .child(error.unwrap_or_else(|| "Sin documento".to_string()))
                            .into_any_element(),
                    }),
            );

        let next_pane = div()
            .flex_1()
            .min_h_0()
            .flex()
            .flex_col()
            .border_t_1()
            .border_color(border)
            .child(div().text_sm().child("Siguiente"))
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .justify_center()
                    .items_center()
                    .p_8()
                    .child(match next_image {
                        Some(image) => img(image).object_fit(ObjectFit::Contain).into_any_element(),
                        None => div()
                            .text_sm()
                            .child(if total == 0 {
                                String::new()
                            } else {
                                "Fin de la presentacion".to_string()
                            })
                            .into_any_element(),
                    }),
            );

        div()
            .flex()
            .flex_col()
            .gap_3()
            .p_4()
            .size_full()
            .bg(bg)
            .text_color(fg)
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(|this, _: &FirstPage, _window, cx| this.apply(cx, Deck::first)))
            .on_action(cx.listener(|this, _: &PrevPage, _window, cx| this.apply(cx, Deck::prev)))
            .on_action(cx.listener(|this, _: &NextPage, _window, cx| this.apply(cx, Deck::next)))
            .on_action(cx.listener(|this, _: &LastPage, _window, cx| this.apply(cx, Deck::last)))
            .on_action(
                cx.listener(|this, _: &ToggleTheme, _window, cx| {
                    this.apply(cx, Deck::toggle_theme)
                }),
            )
            .child(format!("Presenter - pagina {}/{}", page + 1, total_label))
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(self.nav_button("Inicio", cx, Deck::first))
                    .child(self.nav_button("Anterior", cx, Deck::prev))
                    .child(self.nav_button("Siguiente", cx, Deck::next))
                    .child(self.nav_button("Final", cx, Deck::last))
                    .child(self.nav_button("Tema", cx, Deck::toggle_theme)),
            )
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .flex_col()
                    .child(current_pane)
                    .child(next_pane),
            )
    }
}

/// Ventana de la audiencia: solo la pagina actual a pantalla completa.
struct AudienceView {
    deck: Entity<Deck>,
    _sub: Subscription,
}

impl AudienceView {
    fn new(deck: Entity<Deck>, cx: &mut Context<Self>) -> Self {
        let _sub = cx.observe(&deck, |_this, _deck, cx| cx.notify());
        Self { deck, _sub }
    }
}

impl Render for AudienceView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (dark, error) = {
            let deck = self.deck.read(cx);
            (deck.dark, deck.error.clone())
        };
        let image = self.deck.update(cx, |deck, _cx| deck.current_image());

        let bg = if dark { rgb(0x000000) } else { rgb(0xffffff) };
        let fg = if dark { rgb(0xffffff) } else { rgb(0x1e1e1e) };

        let content = match image {
            Some(image) => img(image)
                .size_full()
                .object_fit(ObjectFit::Contain)
                .into_any_element(),
            None => div()
                .size_full()
                .flex()
                .justify_center()
                .items_center()
                .child(error.unwrap_or_else(|| "Sin documento".to_string()))
                .into_any_element(),
        };

        div().size_full().bg(bg).text_color(fg).child(content)
    }
}

/// Convierte en el sitio un buffer RGBA premultiplicado (salida de Typst/tiny-skia)
/// a BGRA no premultiplicado (formato interno de `RenderImage`).
/// Equivale a `gpui::swap_rgba_pa_to_bgra`: intercambia R/B y divide por alpha.
fn rgba_to_bgra(data: &mut [u8]) {
    for chunk in data.as_chunks_mut::<4>().0 {
        chunk.swap(0, 2);
        if chunk[3] > 0 {
            let a = chunk[3] as f32 / 255.0;
            chunk[0] = (chunk[0] as f32 / a) as u8;
            chunk[1] = (chunk[1] as f32 / a) as u8;
            chunk[2] = (chunk[2] as f32 / a) as u8;
        }
    }
}

fn main() {
    let path = std::env::args().nth(1);

    application().run(move |cx: &mut App| {
        cx.bind_keys([
            // Flechas / navegación estándar
            KeyBinding::new("right", NextPage, None),
            KeyBinding::new("left", PrevPage, None),
            KeyBinding::new("home", FirstPage, None),
            KeyBinding::new("end", LastPage, None),
            KeyBinding::new("space", NextPage, None),
            // Estilo vim: h/k atrás, l/j adelante, g inicio, G final
            KeyBinding::new("l", NextPage, None),
            KeyBinding::new("j", NextPage, None),
            KeyBinding::new("h", PrevPage, None),
            KeyBinding::new("k", PrevPage, None),
            KeyBinding::new("g", FirstPage, None),
            KeyBinding::new("shift-g", LastPage, None),
            // Tema
            KeyBinding::new("t", ToggleTheme, None),
        ]);

        let deck: Entity<Deck> = cx.new(|_| match &path {
            Some(path) => Deck::load(path),
            None => Deck::with_error("Uso: gpui_spike <fichero.typ>"),
        });

        let presenter_deck = deck.clone();
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                    None,
                    size(px(900.), px(650.)),
                    cx,
                ))),
                ..Default::default()
            },
            move |window, cx| cx.new(|cx| PresenterView::new(presenter_deck, window, cx)),
        )
        .unwrap();

        let audience_deck = deck.clone();
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Fullscreen(Bounds::default())),
                ..Default::default()
            },
            move |_window, cx| cx.new(|cx| AudienceView::new(audience_deck, cx)),
        )
        .unwrap();

        cx.activate(true);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rgba_to_bgra_unpremultiplies() {
        // Opaco: solo cambia el orden de canales.
        let mut data = vec![255, 0, 0, 255];
        rgba_to_bgra(&mut data);
        assert_eq!(data, [0, 0, 255, 255]);

        // Rojo premultiplicado con alpha 128 -> rojo a plena intensidad.
        let mut data = vec![128, 0, 0, 128];
        rgba_to_bgra(&mut data);
        assert_eq!(data, [0, 0, 254, 128]);

        // Totalmente transparente: no se toca.
        let mut data = vec![0, 0, 0, 0];
        rgba_to_bgra(&mut data);
        assert_eq!(data, [0, 0, 0, 0]);

        // Caso general premultiplicado (cada canal <= alpha).
        let mut data = vec![10, 20, 30, 40];
        rgba_to_bgra(&mut data);
        assert_eq!(data, [191, 127, 63, 40]);
    }
}
