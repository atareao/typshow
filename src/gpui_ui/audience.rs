//! Ventana de la audiencia: la página actual a pantalla completa.

use gpui::{
    Context, Entity, FocusHandle, KeyDownEvent, MouseButton, ObjectFit, Render, Subscription,
    Window, div, img, prelude::*, px,
};

use super::deck::Deck;
use super::keys::apply_nav;
use super::theme::{FONT_SIZE_BODY, palette};

/// Vista de la ventana de audiencia (pantalla completa).
///
/// Gestiona su propio foco y teclado: GPUI despacha los eventos por la ventana
/// activa, y esta (fullscreen) suele tenerlo, así que necesita sus handlers.
pub struct AudienceView {
    deck: Entity<Deck>,
    focus_handle: FocusHandle,
    _sub: Subscription,
}

impl AudienceView {
    /// Crea la vista de audiencia para la `deck` compartida.
    pub fn new(deck: Entity<Deck>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        focus_handle.focus(window, cx);
        let _sub = cx.observe(&deck, |_this, _deck, cx| cx.notify());
        Self {
            deck,
            focus_handle,
            _sub,
        }
    }

    fn run(&self, action: fn(&mut Deck), cx: &mut Context<Self>) {
        self.deck.update(cx, |deck, cx| {
            action(deck);
            cx.notify();
        });
    }
}

impl Render for AudienceView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (dark, error, compiling) = {
            let deck = self.deck.read(cx);
            (deck.dark, deck.error.clone(), deck.loading)
        };
        let (image, loading) = self.deck.update(cx, |deck, cx| {
            let image = deck.current_image(cx);
            let loading = image.is_none() && (deck.loading || deck.is_pending(deck.page));
            (image, loading)
        });
        let p = palette(dark);
        let message = if compiling {
            "Compilando…".to_string()
        } else if loading && error.is_none() {
            "Cargando…".to_string()
        } else {
            error.unwrap_or_else(|| "Sin documento".to_string())
        };

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
                .text_size(px(FONT_SIZE_BODY))
                .text_color(p.fg)
                .child(message)
                .into_any_element(),
        };

        div()
            .size_full()
            .bg(p.view_bg)
            .text_color(p.fg)
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                if event.keystroke.key == "escape" {
                    this.deck.update(cx, |deck, cx| {
                        deck.projection = false;
                        cx.notify();
                    });
                    window.remove_window();
                } else {
                    apply_nav(&this.deck, event, cx);
                }
            }))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _e, _w, cx| this.run(Deck::prev, cx)),
            )
            .on_mouse_down(
                MouseButton::Right,
                cx.listener(|this, _e, _w, cx| this.run(Deck::next, cx)),
            )
            .child(content)
    }
}
