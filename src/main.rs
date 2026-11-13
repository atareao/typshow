//! Punto de entrada de Typshow (UI GPUI).
//!
//! Abre la ventana del presentador; desde ella se abre la ventana de audiencia
//! a pantalla completa al proyectar. Ambas comparten el estado de la
//! presentación (`Deck`) mediante una `Entity` de GPUI y renderizan páginas
//! Typst como imagen.

use gpui::{
    App, AppContext as _, Bounds, Entity, TitlebarOptions, WindowBounds, WindowOptions, px, size,
};
use gpui_platform::application;

use typshow::gpui_ui::deck::Deck;
use typshow::gpui_ui::icons;
use typshow::gpui_ui::presenter::{PresenterView, window_title};

/// Mensaje de uso que se muestra cuando falta el fichero `.typ`.
const USAGE: &str = "Uso: typshow <fichero.typ>";

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let path = std::env::args().nth(1);

    // `-h`/`--help` se resuelve antes de abrir ninguna ventana para que
    // funcione también sin servidor gráfico. El usuario lo pide explícitamente,
    // así que el uso va a stdout (stderr se reserva para el argumento faltante).
    if matches!(path.as_deref(), Some("-h" | "--help")) {
        println!("{USAGE}");
        return;
    }

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
            None => Deck::with_error(USAGE),
        });

        let presenter_deck = deck.clone();
        let title = window_title(path.as_deref());
        let opened = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                    None,
                    size(px(900.), px(650.)),
                    cx,
                ))),
                // `app_id` estable para reglas de compositor (p. ej. Niri).
                app_id: Some("com.atareao.typshow.presenter".to_string()),
                titlebar: Some(TitlebarOptions {
                    title: Some(title.into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            move |window, cx| cx.new(|cx| PresenterView::new(presenter_deck, window, cx)),
        );
        if let Err(error) = opened {
            tracing::error!("no se pudo abrir la ventana del presentador: {error}");
            cx.quit();
            return;
        }

        cx.activate(true);
    });
}
