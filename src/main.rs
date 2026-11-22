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
use typshow::gpui_ui::quits_app_on_close;

/// Mensaje de uso que se muestra cuando falta el fichero `.typ` o se pide ayuda.
const USAGE: &str =
    "Uso: typshow <fichero.typ>\n     typshow guion-embed <guion.md> <presentacion.typ>";
/// Uso específico del subcomando de conversión de guion.
const GUION_EMBED_USAGE: &str = "Uso: typshow guion-embed <guion.md> <presentacion.typ>";

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let args: Vec<String> = std::env::args().collect();

    // `guion-embed` es una utilidad de línea de comandos: convierte un guion
    // Markdown en bloques embebidos del `.typ` y termina sin abrir ventanas.
    if args.get(1).map(String::as_str) == Some("guion-embed") {
        match (args.get(2), args.get(3)) {
            (Some(script), Some(typ)) => match typshow::guion_embed_files(script, typ) {
                Ok(written) => println!("Bloques de guion escritos: {written}"),
                Err(error) => {
                    eprintln!("Error convirtiendo el guion: {error}");
                    std::process::exit(1);
                }
            },
            _ => {
                eprintln!("{GUION_EMBED_USAGE}");
                std::process::exit(2);
            }
        }
        return;
    }

    let path = args.get(1).cloned();

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
            None => Deck::no_document(USAGE),
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
        let presenter = match opened {
            Ok(handle) => handle,
            Err(error) => {
                tracing::error!("no se pudo abrir la ventana del presentador: {error}");
                cx.quit();
                return;
            }
        };

        // Cerrar la ventana del presentador termina la app (y con ella la audiencia,
        // si estaba proyectando). Cerrar solo la ventana de audiencia NO termina la
        // app: eso lo gestiona `PresenterView` desactivando la proyección.
        //
        // La decisión vive en el predicado puro `quits_app_on_close` (testeable sin
        // ventanas); aquí solo se cablea al ciclo de vida real de GPUI.
        //
        // `detach` es necesario: `Subscription` se da de baja al caer, y este
        // closure de arranque retorna en cuanto se monta la ventana, así que sin
        // desligarla el handler se desregistraría antes de poder dispararse.
        let presenter_id = presenter.window_id();
        cx.on_window_closed(move |cx, window_id| {
            if quits_app_on_close(window_id, presenter_id) {
                cx.quit();
            }
        })
        .detach();

        cx.activate(true);
    });
}
