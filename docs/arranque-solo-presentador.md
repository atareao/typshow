# Arranque solo con el presentador, cierre conjunto y tema persistente

## Qué es

Cambios de arranque y preferencias de la app Typshow (issue #42):

- Al abrir la app se abre **solo** la ventana del presentador (antes también se
  abría la de audiencia).
- Cerrar el presentador **cierra toda la app** (incluida la audiencia si estaba
  proyectando).
- El tema claro/oscuro **persiste** entre sesiones aunque la app se arranque
  **sin documento** (el caso del lanzador `.desktop`, que usa `Exec=typshow`).

## Ficheros implicados

- `src/gpui_ui/deck.rs` — `Deck::empty()` arranca con `projection: false`;
  helper `apply_saved_theme`; constructores `no_document`/`no_document_with`.
- `src/gpui_ui/mod.rs` — predicado puro `quits_app_on_close`.
- `src/main.rs` — usa `Deck::no_document(USAGE)` sin documento; registra
  `cx.on_window_closed(...).detach()` → `quits_app_on_close` → `cx.quit()`.
- `src/gpui_ui/presenter.rs` — test headless del wiring (`window_close_tests`).

## Cómo funciona

- **Arranque:** sin argumento, `main.rs` construía el deck con `Deck::with_error`
  (ahora `Deck::no_document`), que dejaba `theme_path: None` → el tema nunca se
  guardaba. Ahora `no_document` cablea `config::theme_path()` como `Deck::load`.
  Como `Deck::empty()` arranca con `projection: false`, no se abre la audiencia.
- **Tema:** se lee en `apply_saved_theme` (`config::load_theme_from`) y se guarda
  en `Deck::toggle_theme` (`config::save_theme_to`). Fichero:
  `$XDG_CONFIG_HOME/typshow/theme` o, si no, `$HOME/.config/typshow/theme`,
  con contenido `dark\n` o `light\n`.
- **Cierre:** cerrar la ventana del presentador dispara `on_window_closed`; si el
  `window_id` coincide con el del presentador (`quits_app_on_close`) se llama a
  `cx.quit()`, que termina la app y cierra también la audiencia. Cerrar solo la
  audiencia **no** cierra la app: `PresenterView` desactiva la proyección.

## Variables / configuración

- `XDG_CONFIG_HOME`, `HOME` — ruta del fichero de tema.
- Sin nuevas variables de entorno ni ficheros de config.

## Cómo habilitar / desplegar

Sin cambios de empaquetado. El lanzador `~/.local/share/applications/typshow.desktop`
sigue con `Exec=typshow`.

## Decisiones

- El tema es una preferencia **de la app**, no del documento: se cablea en
  cualquier arranque (con o sin `.typ`).
- `.detach()` es necesario en la `Subscription` de `on_window_closed`: si el
  temporal se dropea al final de la sentencia, el handler se desuscribe y nunca
  dispara.
- La decisión de cerrar la app vive en el predicado puro `quits_app_on_close`
  (testeable sin plataforma).

## Verificación

- `cargo test` (188 verdes), `cargo clippy --all-targets -- -D warnings`,
  `cargo fmt --all -- --check`.
- Manual (Niri, arranque como el `.desktop`, sin argumentos): solo se abre el
  presentador; abre con el tema guardado; la tecla `t` persiste el tema.

## Follow-ups

- Verificar manualmente el cierre con proyección activa (no se pudo automatizar:
  el dispositivo de ratón de `ydotool` no responde en la sesión).
- Tras el merge del issue #42: `openspec archive arranque-solo-presentador` y
  marcar `tasks.md` 4.3.
