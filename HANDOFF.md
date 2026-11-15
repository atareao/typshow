# HANDOFF — Typshow

## Estado actual

Issue #42 **implementado, verificado y mergeado** a `development` (PR #43). El change
OpenSpec está archivado (`openspec/changes/archive/2026-11-15-arranque-solo-presentador/`)
y `plans/PLAN-008-ARCHIVED.md`. `cargo test` 188 verdes, clippy (`-D warnings`) y fmt
limpios. Pendiente **solo**: release patch (PR `development` → `main`) y la verificación
manual del cierre con la proyección activa.

## Completado (sesión 2026-11-15)

- `Deck::empty()` pasa a `projection: false` (arranque sin documento abre solo el
  presentador).
- Nuevo `Deck::no_document()` que cablea `config::theme_path()` (antes, sin
  documento, `theme_path: None` → el tema nunca se guardaba).
- `main.rs`: `on_window_closed` → predicado `quits_app_on_close` → `cx.quit()`
  (cerrar el presentador cierra la app entera).
- Tests nuevos: predicado (`gpui_ui::tests`) y wiring headless
  (`presenter::window_close_tests`).
- Artefactos: issue #42, PR #43, `plans/PLAN-008-ARCHIVED.md`,
  `docs/arranque-solo-presentador.md` y el change archivado en
  `openspec/changes/archive/2026-11-15-arranque-solo-presentador/`.

## Tarea pendiente (relevo)

- Verificar manualmente el cierre con el presentador y la **proyección activa**.
- Preparar release patch: PR `development` → `main` (dispara `release-prepare.yml`).

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
