//! Binario de la UI GPUI de Typshow (coexiste con el binario egui `typshow`).
//!
//! Abre dos ventanas nativas (presenter + audiencia) que comparten el estado de
//! la presentación (`Deck`) mediante una `Entity` de GPUI y renderizan páginas
//! Typst como imagen.

use gpui::{App, AppContext as _, Bounds, Entity, WindowBounds, WindowOptions, px, size};
use gpui_platform::application;

use typshow::gpui_ui::deck::Deck;
use typshow::gpui_ui::icons;
use typshow::gpui_ui::presenter::PresenterView;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let path = std::env::args().nth(1);

    application().run(move |cx: &mut App| {
        // La fuente de iconos debe estar registrada antes de abrir ventanas: los
        // controles pintan cada glifo con `font_family("Phosphor")`, y esa
        // familia solo existe si se ha cargado aquí.
        if let Err(error) = icons::register(cx) {
            tracing::error!("no se pudo registrar la fuente de iconos Phosphor: {error}");
            cx.quit();
            return;
        }

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
                // `app_id` estable para reglas de compositor (p. ej. Niri).
                app_id: Some("com.atareao.typshow.presenter".to_string()),
                ..Default::default()
            },
            move |window, cx| cx.new(|cx| PresenterView::new(presenter_deck, window, cx)),
        )
        .unwrap();

        cx.activate(true);
    });
}
