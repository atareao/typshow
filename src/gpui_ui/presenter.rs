//! Ventana del presentador: página actual + siguiente, navegación y tema.

use gpui::{
    AppContext, Context, Entity, FocusHandle, MouseButton, ObjectFit, Render, Subscription, Window,
    div, img, prelude::*,
};

use super::actions::{FirstPage, LastPage, NextPage, PrevPage, ToggleTheme};
use super::deck::Deck;
use super::theme::palette;

/// Ejecuta `action` sobre el `Deck` compartido y notifica a los observers.
fn run<C: AppContext>(deck: &Entity<Deck>, action: fn(&mut Deck), cx: &mut C) {
    deck.update(cx, |deck, cx| {
        action(deck);
        cx.notify();
    });
}

/// Vista de la ventana del presentador.
pub struct PresenterView {
    deck: Entity<Deck>,
    focus_handle: FocusHandle,
    _sub: Subscription,
}

impl PresenterView {
    /// Crea la vista del presentador para la `deck` compartida.
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

    fn nav_button(
        &self,
        label: &'static str,
        fg: gpui::Rgba,
        bg: gpui::Rgba,
        action: fn(&mut Deck),
    ) -> impl IntoElement {
        let deck = self.deck.clone();
        div()
            .px_4()
            .py_2()
            .bg(bg)
            .rounded_md()
            .text_color(fg)
            .child(label)
            .on_mouse_down(MouseButton::Left, move |_event, _window, cx| {
                run(&deck, action, cx);
            })
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

        let p = palette(dark);
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
            .border_color(p.border)
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
            .bg(p.bg)
            .text_color(p.fg)
            .track_focus(&self.focus_handle)
            .on_action(
                cx.listener(|this, _: &FirstPage, _window, cx| run(&this.deck, Deck::first, cx)),
            )
            .on_action(
                cx.listener(|this, _: &PrevPage, _window, cx| run(&this.deck, Deck::prev, cx)),
            )
            .on_action(
                cx.listener(|this, _: &NextPage, _window, cx| run(&this.deck, Deck::next, cx)),
            )
            .on_action(
                cx.listener(|this, _: &LastPage, _window, cx| run(&this.deck, Deck::last, cx)),
            )
            .on_action(cx.listener(|this, _: &ToggleTheme, _window, cx| {
                run(&this.deck, Deck::toggle_theme, cx)
            }))
            .child(format!("Presenter - pagina {}/{total_label}", page + 1))
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(self.nav_button("Inicio", p.on_accent, p.accent, Deck::first))
                    .child(self.nav_button("Anterior", p.on_accent, p.accent, Deck::prev))
                    .child(self.nav_button("Siguiente", p.on_accent, p.accent, Deck::next))
                    .child(self.nav_button("Final", p.on_accent, p.accent, Deck::last))
                    .child(self.nav_button("Tema", p.on_accent, p.accent, Deck::toggle_theme)),
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
