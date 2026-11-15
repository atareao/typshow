# HANDOFF — Typshow

## Estado actual

En `development`: issue #42 (release **v0.4.0** publicada) e issue #46 mergeado
(PR #47) con el change OpenSpec archivado. `cargo test` 194 + 2 integración verdes,
clippy (`-D warnings`) y fmt limpios. Pendiente: **release del #46** (bump minor
0.5.0, PR `development` → `main`) y la verificación manual del clic sobre «Salir»
con proyección activa.

## Completado (sesiones 2026-11-15)

### Issue #42 — arranque solo con el presentador, cierre conjunto y tema (release v0.4.0)
- `Deck::empty()` → `projection: false`; `Deck::no_document()` cablea el tema;
  `main.rs` `on_window_closed` → `quits_app_on_close` → `cx.quit()`.

### Issue #46 — botón «Salir» y atajo Ctrl+Q (PR #47)
- `keys::quit_shortcut(key, control)` (predicado puro; ignora Shift/Alt/Super a propósito).
- `icons::SIGN_OUT` = `'\u{E42A}'` (en `ALL` y tabla dorada).
- `PresenterView::exit_app` = `window.remove_window()` + `cx.quit()`; botón `btn-salir`
  en la barra superior; `dispatch_key` gestiona `Ctrl+Q` antes de editar/navegar.
- Tests headless: Ctrl+Q, Ctrl+Q en edición, y clic del botón con proyección activa.

## Tarea pendiente (relevo)

- Verificar manualmente el clic sobre «Salir» con la **proyección activa**.
- Preparar release del #46: PR `development` → `main` (bump minor → 0.5.0).

## Follow-ups no bloqueantes

- Editor: undo/redo, navegación por palabras, panel de notas redimensionable.
- Rama `feature/sync-renderer` (1 commit, sin PR): decidir PR o borrar.
- Change OpenSpec `gpui-spike` sigue activo (spike, no archivado).
- `Ctrl+Q` solo en el presentador; valorar extenderlo a la ventana de audiencia.

## Notas de reproducción / entorno

- Lanzar: `target/debug/typshow <path.typ>` o sin argumento (solo presentador).
- `ydotool key` usa **keycodes numéricos** (Esc=1, t=20, q=16, ctrl=29).
  `Ctrl+Q` = `ydotool key 29:1 16:1 16:0 29:0`. El ratón (`mousemove`/`click`) NO
  funciona en esta sesión.
- Captura: `grim -o DP-1 <png>`. Overview de Niri: `niri msg action toggle-overview`.
- `app_id`: presentador `com.atareao.typshow.presenter`, audiencia
  `com.atareao.typshow.audience`.
- CI clippy = **1.100.0**: `rustup run 1.100.0 cargo clippy --all-targets -- -D warnings`.
