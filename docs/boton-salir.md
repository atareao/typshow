# Botón «Salir» y atajo Ctrl+Q

## Qué es

Añade una forma explícita de terminar Typshow (issue #46):

- Un botón **«Salir»** en la barra superior de controles del presentador, junto a
  «Tema».
- El atajo **`Ctrl+Q`**, activo también mientras se edita una nota.

Ambos cierran **toda** la aplicación (presentador y audiencia). El cierre es
**directo**, sin diálogo de confirmación.

## Ficheros implicados

- `src/gpui_ui/keys.rs` — predicado puro `quit_shortcut(key, control)` (`control && key == "q"`).
- `src/gpui_ui/icons.rs` — icono `SIGN_OUT` = `'\u{E42A}'` (añadido a `ALL` y a la tabla dorada del test).
- `src/gpui_ui/presenter.rs` — `PresenterView::exit_app`; botón `btn-salir`; rama de
  `Ctrl+Q` en `dispatch_key`; tests en `mod exit_tests`.

## Cómo funciona

- `PresenterView::exit_app(window, cx)` hace **dos** cosas: `window.remove_window()`
  (cierra la ventana del presentador, lo que notifica el cierre y desactiva la
  proyección) y `cx.quit()` (pide el fin de la app). Así la salida **no depende** de
  que exista el handler `on_window_closed` de `main.rs` (issue #42); ese handler sigue
  presente y es idempotente.
- El botón «Salir» tiene un listener propio (`on_mouse_down` → `exit_app`) porque la
  acción no es de `Deck` (no encaja en los helpers `action_button`/`nav_button`).
- `dispatch_key` comprueba `quit_shortcut(keystroke.key, keystroke.modifiers.control)`
  **antes** de la bifurcación edición/navegación y, si coincide, llama a `exit_app` y
  `cx.stop_propagation()`. Por eso `Ctrl+Q` funciona también editando (no llega al
  editor). Nota: `quit_shortcut` **ignora Shift/Alt/Super** a propósito (en edición,
  `Ctrl+<letra>` distinta de a/c/x/v se trata como `Ignore`), así que `Ctrl+Shift+Q`
  también sale.

## Variables / configuración

- Ninguna. Sin cambios de entorno ni de ficheros de config.

## Cómo habilitar / desplegar

Sin cambios de empaquetado. Se añadió `debug_selector("btn-salir")` en el botón para
poder simular el clic en tests headless; es un no-op fuera de builds con `test-support`
(no afecta al binario de producción).

## Decisiones

- **Cierre directo**, sin confirmación por cambios sin guardar (decisión del usuario).
- `exit_app` usa `remove_window()` **y** `cx.quit()`: ruta robusta (no depende del
  handler del binario) y observable en tests (los `window_closed_observers` sí se
  disparan; `cx.quit()` es no-op en `TestPlatform`).
- `Ctrl+Q` solo se gestiona en la ventana del presentador; la audiencia se cierra vía
  `cx.quit()`/handler.

## Verificación

- `cargo test` (194 lib + 2 integración), `cargo clippy --all-targets -- -D warnings`,
  `cargo fmt --all -- --check`.
- Tests: `keys::quit_shortcut_matches_ctrl_q`, `keys::quit_shortcut_ignores_shift_on_purpose`,
  `presenter::exit_tests::{ctrl_q_closes_the_presenter_window, ctrl_q_closes_the_window_while_editing_a_note, exit_button_closes_the_app_with_projection_active}`.
- Manual (Niri): `Ctrl+Q` cierra la app; el botón «Salir» aparece en la barra superior.

## Follow-ups

- Verificación manual del clic sobre «Salir» con la proyección activa (automatizado ya
  en headless; el dispositivo de ratón de `ydotool` no responde en la sesión).
- Si se quisiera, extender `Ctrl+Q` a la ventana de audiencia.
