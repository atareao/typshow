//! Binario de la UI GPUI de Typshow (coexiste con el binario egui `typshow`).
//!
//! Abre dos ventanas nativas (presenter + audiencia) que comparten el estado de
//! la presentación (`Deck`) mediante una `Entity` de GPUI y renderizan páginas
//! Typst como imagen.

use gpui::{
    App, AppContext as _, Bounds, Entity, KeyBinding, WindowBounds, WindowOptions, px, size,
};
use gpui_platform::application;

use typshow::gpui_ui::actions::{FirstPage, LastPage, NextPage, PrevPage, ToggleTheme};
use typshow::gpui_ui::audience::AudienceView;
use typshow::gpui_ui::deck::Deck;
use typshow::gpui_ui::presenter::PresenterView;

fn main() {
    let path = std::env::args().nth(1);

    application().run(move |cx: &mut App| {
        cx.bind_keys([
            // Navegación estándar
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
            None => Deck::with_error("Uso: typshow-gpui <fichero.typ>"),
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
            move |window, cx| cx.new(|cx| AudienceView::new(audience_deck, window, cx)),
        )
        .unwrap();

        cx.activate(true);
    });
}
