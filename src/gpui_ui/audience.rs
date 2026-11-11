//! Ventana de la audiencia: la página actual a pantalla completa.

use gpui::{
    AppContext, Context, Entity, FocusHandle, ObjectFit, Render, Subscription, Window, div, img,
    prelude::*,
};

use super::actions::{FirstPage, LastPage, NextPage, PrevPage, ToggleTheme};
use super::deck::Deck;
use super::theme::palette;

/// Vista de la ventana de audiencia (pantalla completa).
///
/// Registra las mismas acciones de teclado que el presentador: GPUI despacha
/// las acciones por la ruta de foco de la ventana activa, y esta ventana
/// (fullscreen) suele tener el foco, así que necesita sus propios handlers.
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

    fn run<C: AppContext>(&self, action: fn(&mut Deck), cx: &mut C) {
        self.deck.update(cx, |deck, cx| {
            action(deck);
            cx.notify();
        });
    }
}

impl Render for AudienceView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (dark, error) = {
            let deck = self.deck.read(cx);
            (deck.dark, deck.error.clone())
        };
        let image = self.deck.update(cx, |deck, _cx| deck.current_image());
        let p = palette(dark);

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

        div()
            .size_full()
            .bg(p.bg)
            .text_color(p.fg)
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(|this, _: &FirstPage, _window, cx| this.run(Deck::first, cx)))
            .on_action(cx.listener(|this, _: &PrevPage, _window, cx| this.run(Deck::prev, cx)))
            .on_action(cx.listener(|this, _: &NextPage, _window, cx| this.run(Deck::next, cx)))
            .on_action(cx.listener(|this, _: &LastPage, _window, cx| this.run(Deck::last, cx)))
            .on_action(
                cx.listener(|this, _: &ToggleTheme, _window, cx| this.run(Deck::toggle_theme, cx)),
            )
            .child(content)
    }
}
