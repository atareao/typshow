# Handoff — typshow — 2026-11-13 05:40

## Objetivo
Migrar la UI de Typshow de `egui`/`eframe` a `GPUI` (issue #12) por fases. **PR #14 MERGEADO a `development`** (hito 3.12/M4). Quedan las fases finales, fases 5-7.

## Estado
- **PR #14** (`feature/gpui-migration` → `development`): **MERGED** (hito M4). CI verde (tests, fmt, clippy).
- Hecho: Fases 1/3 (doble ventana, render asíncrono, caché LRU), 3.8–3.12/M4 (editor `TextEdit` + `EntityInputHandler`, verificado manualmente en Niri), 2.5 (apertura asíncrona de `.typ`).
- Hecho: perfil dev optimizado en `Cargo.toml` (`[profile.dev.package."*"] opt-level = 3`).
- Hecho: CI arreglada (deps de sistema de GPUI + caché de Cargo) y cero warnings de clippy.
- **Hecho ahora (tarea 2.2 del change `openspec/changes/migracion-gpui/`) en la rama `feature/gpui-decouple-notes`:** `src/notes.rs` ya no depende de `egui`. El pintado (`draw` + `format_inline` + 4 tests) se movió al nuevo `src/notes_egui.rs`; se declara `mod notes_egui;` en `src/lib.rs` y se actualizó el call-site en `src/presenter.rs`. Commits `e9c29b2` (refactor) y `7084178` (docs).
- **PR #16** (`feature/gpui-decouple-notes` → `development`): ABIERTO, pendiente de merge — https://github.com/atareao/typshow/pull/16
- Verificación en verde: `cargo test --lib` → 149 passed; `cargo fmt --all --check` limpio; `cargo clippy --all-targets -- -D warnings` (toolchain 1.99.1) → 0 warnings; `cargo build --bin typshow` OK. Auditado por `@rust-reviewer`: APRUEBA.
- Documentación de estado al día: 2.2 marcada en `openspec/changes/migracion-gpui/tasks.md` y `plans/PLAN-002.md` actualizado.

## Pendiente
- **Fases 5-7** (PLAN-002 sigue EN CURSO):
  - Tema/iconos Adwaita en GPUI y sustituir `egui-phosphor`.
  - GPUI como binario por defecto `typshow` y retirada de `egui`/`eframe`/`egui-phosphor`.
  - Cierre con `docs/migracion-gpui.md` + verificación final.
- Decidir qué hacer con el **PR #13** (`feature/gpui-spike`), que sigue ABIERTO.
- Decidir si se borra la rama `feature/gpui-migration` (ya mergeada) o se reutiliza para las fases 5-7.

## Riesgos y notas
- Para el binario definitivo usar `cargo build --release`.
- El clippy del runner es más nuevo que el local: verificar lints nuevos con `rustup run 1.99.1 cargo clippy --all-targets -- -D warnings`.
- El subagente `rust-writer` está fallando en silencio (no ejecuta escrituras ni responde): usar `backend-dev` como sustituto.
- El binario `typshow-gpui` sigue compilando la lib con `egui`; lo resolverá la fase 6.
