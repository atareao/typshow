# HANDOFF — Typshow

## Estado actual

Cambios del issue #42 **implementados y verificados, sin commitear** en `development`.
Código OK, `cargo test` 188 verdes, clippy (`-D warnings`) y fmt limpios, y change
OpenSpec `arranque-solo-presentador` validado. Falta rama/commit/PR.

## Completado (sesión 2026-11-15)

- `Deck::empty()` pasa a `projection: false` (arranque sin documento abre solo el
  presentador).
- Nuevo `Deck::no_document()` que cablea `config::theme_path()` (antes, sin
  documento, `theme_path: None` → el tema nunca se guardaba).
- `main.rs`: `on_window_closed` → predicado `quits_app_on_close` → `cx.quit()`
  (cerrar el presentador cierra la app entera).
- Tests nuevos: predicado (`gpui_ui::tests`) y wiring headless
  (`presenter::window_close_tests`).
- Artefactos: issue #42, `plans/PLAN-008.md`, `docs/arranque-solo-presentador.md`
  y change OpenSpec `openspec/changes/arranque-solo-presentador/`.

## Tarea pendiente (relevo)

- Verificar manualmente el cierre con el presentador y la **proyección activa**.
- Crear rama `feature/arranque-solo-presentador` desde `development`, commit
  (gitmoji) y PR que cierre el issue #42.
- Tras el merge: `openspec archive arranque-solo-presentador`, marcar `tasks.md`
  4.3 y preparar release (patch).

## Follow-ups no bloqueantes

- Editor: undo/redo, navegación por palabras, panel de notas redimensionable.
- Rama `feature/sync-renderer` (1 commit, sin PR): decidir PR o borrar.
- Change OpenSpec `gpui-spike` sigue activo (spike, no archivado).

## Notas de reproducción / entorno

- Lanzar: `target/debug/typshow <path.typ>` o sin argumento (solo presentador).
- `ydotool key` usa **keycodes numéricos** (Esc=1, t=20, l=38); el ratón
  (`mousemove`/`click`) NO funciona en esta sesión.
- Captura: `grim -o DP-1 <png>`. Overview de Niri: `niri msg action toggle-overview`.
- `app_id`: presentador `com.atareao.typshow.presenter`, audiencia
  `com.atareao.typshow.audience`.
- CI clippy = **1.100.0**: `rustup run 1.100.0 cargo clippy --all-targets -- -D warnings`.
