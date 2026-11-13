# Handoff — typshow — 2026-11-13

## Objetivo
Migrar la UI de Typshow de `egui`/`eframe` a `GPUI` (issue #12). **Migración terminada.**

## Estado
- **Fases 1-7 completadas.** `typshow` es el único binario (GPUI); `egui`, `eframe`,
  `egui-phosphor`, `winit` y `parking_lot` retirados de `Cargo.toml`.
- Change `migracion-gpui` **archivado** (`openspec/changes/archive/2026-11-13-migracion-gpui/`);
  spec viva en `openspec/specs/gpui-ui/spec.md`.
- `plans/PLAN-002-ARCHIVED.md` cerrado. `docs/migracion-gpui.md` reescrito;
  `docs/gpui-spike.md` marcado como registro histórico.
- Verificación en verde: `cargo build`, `cargo build --release`, `cargo test` (166 lib + 2
  integración), `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`.
- Rama de trabajo: `feature/gpui-default-binary` (cambios **sin commitear**).

## Pendiente
- **Verificación visual en Niri**: tema, iconos, markdown de las notas y Esc/proyección.
- Follow-ups del editor: undo/redo, navegación por palabras, panel de notas redimensionable,
  decidir `MdElement::EmptyLine` (sin productor).
- Commit + PR de `feature/gpui-default-binary` → `development`.
- Decidir qué hacer con el **PR #13** (`feature/gpui-spike`), que sigue ABIERTO.

## Riesgos y notas
- Para el binario definitivo usar `cargo build --release`.
- El clippy del runner es más nuevo que el local: verificar lints nuevos con
  `rustup run 1.99.1 cargo clippy --all-targets -- -D warnings`.
