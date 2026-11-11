# HANDOFF — Typshow

## Estado
Migración de la UI de Typshow de `egui`/`eframe` a **GPUI** (issue #12), por fases.
- **Fase 1 COMPLETADA** (estructura + núcleo + presenter + audiencia básicos).
- PRs abiertos: **#13** (spike, `feature/gpui-spike`) y **#14** (migración Fase 1,
  borrador, `feature/gpui-migration`) → `development`.

## Rama actual
`feature/gpui-migration` (commit `ed10a81`). Coexisten dos binarios:
`typshow` (egui, intacto) y `typshow-gpui` (nuevo).

## Hecho
- `src/gpui_ui/`: `deck.rs` (estado + render Typst→RenderImage con caché),
  `presenter.rs`, `audience.rs`, `theme.rs`, `actions.rs`, `mod.rs`.
- `src/bin/typshow_gpui.rs` + `[[bin]]` en `Cargo.toml`.
- `openspec/changes/migracion-gpui/` (válido `--strict`) y `plans/PLAN-002.md`.
- Auditoría `@rust-reviewer` aplicada: nav por teclado en la audiencia + delegación en
  `gpui::swap_rgba_pa_to_bgra`.
- Verificado: `cargo build`/`test` verdes (68 lib + 1 spike + 2 pipeline), clippy limpio
  en `gpui_ui`, arranque real con la presentación de ejemplo.

## Siguiente (Fase 3 — presenter)
1. Abrir fichero (`rfd`, filtrar `.typ`).
2. Toggle de proyección (mostrar/ocultar audiencia).
3. Panel de notas (ver/editar/guardar/reimportar) reutilizando `Notes`/`NotesDb`.
4. Estado disabled de la barra de navegación; navegación por clic.
5. Fase 4: cierre con Esc y audiencia en monitor secundario (Niri).

## Comandos útiles
- Build/run: `cargo build --bin typshow-gpui && ./target/debug/typshow-gpui <file.typ>`
- Matar: `pkill -x typshow_gpui` (NO `pkill -f`).
- Tests: `cargo test` · Lint: `cargo clippy --bin typshow-gpui`
